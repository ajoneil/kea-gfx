use crate::gpu::{
    buffer::{AllocatedBuffer, Buffer},
    command::CommandPool,
    descriptor_set::{DescriptorSetLayout, DescriptorSetLayoutBinding},
    device::Device,
    pipeline::PipelineLayout,
    rt::acceleration_structure::{
        Aabb, AccelerationStructure, AccelerationStructureDescription, Geometry,
    },
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

        Self::create_pipeline(&device);

        PathTracer {
            device,
            command_pool,
            tl_acceleration_structure,
            bl_acceleration_structure,
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

    fn create_pipeline(device: &Arc<Device>) {
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

        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), &bindings);
        let pipeline_layout = PipelineLayout::new(device.clone(), &[descriptor_set_layout]);
    }
}
