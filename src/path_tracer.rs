use crate::gpu::{
    buffer::{AllocatedBuffer, Buffer},
    command::CommandPool,
    descriptor_set::{
        DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding,
    },
    device::Device,
    pipeline::{
        Pipeline, PipelineDescription, PipelineLayout, PipelineShaderStage,
        RayTracingPipelineDescription,
    },
    rt::acceleration_structure::{
        Aabb, AccelerationStructure, AccelerationStructureDescription, Geometry,
    },
    shaders::ShaderModule,
};
use ash::vk;
use glam::{vec3, Vec3};
use gpu_allocator::MemoryLocation;
use std::{mem, sync::Arc};

pub struct PathTracer {
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    tl_acceleration_structure: AccelerationStructure,
    bl_acceleration_structure: AccelerationStructure,
    pipeline: Pipeline,
    descriptor_set_layouts: [DescriptorSetLayout; 1],
    descriptor_sets: Vec<DescriptorSet>,
}

struct Sphere {
    position: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn aabb(&self) -> Aabb {
        let Sphere { position, radius } = self;

        Aabb {
            min: vec3(
                position.x - radius,
                position.y - radius,
                position.z - radius,
            ),
            max: vec3(
                position.x + radius,
                position.y + radius,
                position.z + radius,
            ),
        }
    }
}

impl PathTracer {
    pub fn new(device: Arc<Device>) -> PathTracer {
        let command_pool = Arc::new(CommandPool::new(device.queues().graphics()));
        let (tl_acceleration_structure, bl_acceleration_structure) =
            Self::build_acceleration_structure(&device, &command_pool);

        let (pipeline, descriptor_set_layouts) = Self::create_pipeline(&device);
        let descriptor_sets = Self::create_descriptor_sets(device.clone(), &descriptor_set_layouts);

        PathTracer {
            device,
            command_pool,
            tl_acceleration_structure,
            bl_acceleration_structure,
            pipeline,
            descriptor_set_layouts,
            descriptor_sets,
        }
    }

    fn build_acceleration_structure(
        device: &Arc<Device>,
        command_pool: &Arc<CommandPool>,
    ) -> (AccelerationStructure, AccelerationStructure) {
        let spheres = [Sphere {
            position: vec3(0.0, 0.0, 1.0),
            radius: 0.5,
        }];

        let (spheres_buffer, aabbs_buffer) = Self::create_buffers(device, &spheres);
        let geometries = [Geometry::aabbs(&aabbs_buffer)];
        let blas = AccelerationStructureDescription::new(
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            &geometries,
        );

        let build_sizes = blas.build_sizes(device);
        let scratch_buffer = Buffer::new(
            device.clone(),
            build_sizes.build_scratch,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("scratch", MemoryLocation::GpuOnly, true);

        let bl_acceleration_structure_buffer = Buffer::new(
            device.clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("bl acceleration structure", MemoryLocation::GpuOnly, true);

        let bl_acceleration_structure = AccelerationStructure::new(
            device,
            bl_acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
        );

        let cmd = command_pool.allocate_buffer();
        cmd.record(true, |cmd| {
            cmd.build_acceleration_structure(&blas, &bl_acceleration_structure, &scratch_buffer);
        });
        cmd.submit().wait();

        let identity_transform = vk::TransformMatrixKHR {
            matrix: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        };

        let custom_index = 0;
        let mask = 0xff;
        let shader_binding_table_record_offset = 0;
        let flags = vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE
            .as_raw()
            .try_into()
            .unwrap();
        let tlas_instance = vk::AccelerationStructureInstanceKHR {
            transform: identity_transform,
            instance_custom_index_and_mask: vk::Packed24_8::new(custom_index, mask),
            instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                shader_binding_table_record_offset,
                flags,
            ),
            acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                device_handle: bl_acceleration_structure.buffer().device_address(),
            },
        };
        let tlas_buffer = Buffer::new(
            device.clone(),
            mem::size_of_val(&tlas_instance) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("tlas build", MemoryLocation::CpuToGpu, true);
        tlas_buffer.fill(&[tlas_instance]);

        let geometries = [Geometry::instances(&tlas_buffer)];
        let tlas = AccelerationStructureDescription::new(
            vk::AccelerationStructureTypeKHR::TOP_LEVEL,
            &geometries,
        );

        let build_sizes = tlas.build_sizes(device);
        let scratch_buffer = Buffer::new(
            device.clone(),
            build_sizes.build_scratch,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("scratch", MemoryLocation::GpuOnly, true);

        let tl_acceleration_structure_buffer = Buffer::new(
            device.clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("tl acceleration structure", MemoryLocation::GpuOnly, true);

        let tl_acceleration_structure = AccelerationStructure::new(
            device,
            tl_acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::TOP_LEVEL,
        );

        let cmd = command_pool.allocate_buffer();
        cmd.record(true, |cmd| {
            cmd.build_acceleration_structure(&tlas, &tl_acceleration_structure, &scratch_buffer);
        });
        cmd.submit().wait();

        (bl_acceleration_structure, tl_acceleration_structure)
    }

    fn create_buffers(
        device: &Arc<Device>,
        spheres: &[Sphere],
    ) -> (AllocatedBuffer, AllocatedBuffer) {
        let spheres_buffer = Buffer::new(
            device.clone(),
            (mem::size_of::<Sphere>() * spheres.len()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );

        let spheres_buffer = spheres_buffer.allocate("spheres", MemoryLocation::CpuToGpu, true);
        spheres_buffer.fill(spheres);

        let aabbs: Vec<Aabb> = spheres.iter().map(|s: &Sphere| s.aabb()).collect();
        let aabbs_buffer = Buffer::new(
            device.clone(),
            (mem::size_of::<Aabb>() * aabbs.len()) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let aabbs_buffer = aabbs_buffer.allocate("vertices", MemoryLocation::CpuToGpu, true);
        aabbs_buffer.fill(&aabbs);

        (spheres_buffer, aabbs_buffer)
    }

    fn create_pipeline(device: &Arc<Device>) -> (Pipeline, [DescriptorSetLayout; 1]) {
        let bindings = [
            DescriptorSetLayoutBinding::new(
                0,
                vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                1,
                vk::ShaderStageFlags::RAYGEN_KHR,
            ),
            DescriptorSetLayoutBinding::new(
                1,
                vk::DescriptorType::STORAGE_IMAGE,
                1,
                vk::ShaderStageFlags::RAYGEN_KHR,
            ),
        ];

        let descriptor_set_layouts = [DescriptorSetLayout::new(device.clone(), &bindings)];
        let pipeline_layout = PipelineLayout::new(device.clone(), &descriptor_set_layouts);

        let shader_module = ShaderModule::new(device.clone());
        let shader_stages = [
            PipelineShaderStage::new(
                vk::ShaderStageFlags::RAYGEN_KHR,
                &shader_module.entry_point("generate_rays"),
            ),
            PipelineShaderStage::new(
                vk::ShaderStageFlags::MISS_KHR,
                &shader_module.entry_point("ray_miss"),
            ),
            PipelineShaderStage::new(
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
                &shader_module.entry_point("ray_hit"),
            ),
            PipelineShaderStage::new(
                vk::ShaderStageFlags::INTERSECTION_KHR,
                &shader_module.entry_point("intersect_sphere"),
            ),
        ];

        let shader_groups = [
            // generate
            vk::RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                .general_shader(0)
                .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR)
                .build(),
            // miss
            vk::RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                .general_shader(1)
                .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR)
                .build(),
            // sphere hit
            vk::RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(vk::RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP)
                .general_shader(vk::SHADER_UNUSED_KHR)
                .closest_hit_shader(2)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(3)
                .build(),
        ];

        let pipeline_desc = PipelineDescription::RayTracing(RayTracingPipelineDescription::new(
            &shader_stages,
            &shader_groups,
            &pipeline_layout,
        ));
        let pipeline = Pipeline::new(device.clone(), &pipeline_desc);

        (pipeline, descriptor_set_layouts)
    }

    fn create_descriptor_sets(
        device: Arc<Device>,
        layouts: &[DescriptorSetLayout],
    ) -> Vec<DescriptorSet> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool = DescriptorPool::new(device, 1, &pool_sizes);
        descriptor_pool.allocate_descriptor_sets(layouts)
    }
}
