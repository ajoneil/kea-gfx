use ash::vk;
use glam::{vec3, Vec3};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc},
    MemoryLocation,
};
use kea_gpu::{
    core::{
        buffer::{AllocatedBuffer, Buffer},
        command::CommandPool,
        descriptor_set::{
            DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding,
        },
        pipeline::{
            Pipeline, PipelineDescription, PipelineLayout, PipelineShaderStage,
            RayTracingPipelineDescription,
        },
        shaders::ShaderModule,
    },
    device::Device,
    ray_tracing::{
        acceleration_structure::{
            Aabb, AccelerationStructure, AccelerationStructureDescription, Geometry,
        },
        shader_binding_table::{RayTracingShaderBindingTables, ShaderBindingTable},
    },
    Kea,
};
use log::info;
use std::{
    mem::{self, ManuallyDrop},
    slice,
    sync::Arc,
};

pub struct PathTracer {
    kea: Kea,
    _command_pool: Arc<CommandPool>,
    _tl_acceleration_structure: AccelerationStructure,
    _bl_acceleration_structure: AccelerationStructure,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    _descriptor_set_layout: DescriptorSetLayout,
    descriptor_set: DescriptorSet,
    _aabbs_buffer: AllocatedBuffer,
    _spheres_buffer: AllocatedBuffer,
    storage_image: vk::Image,
    storage_image_view: vk::ImageView,
    allocation: ManuallyDrop<Allocation>,
    _shader_binding_tables_buffer: AllocatedBuffer,
    shader_binding_tables: RayTracingShaderBindingTables,
}

#[derive(Debug)]
#[repr(C)]
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
    pub fn new(kea: Kea) -> PathTracer {
        let command_pool = CommandPool::new(kea.device().graphics_queue());
        let (tl_acceleration_structure, bl_acceleration_structure, spheres_buffer, aabbs_buffer) =
            Self::build_acceleration_structure(&kea);
        let (pipeline, pipeline_layout, descriptor_set_layout) =
            Self::create_pipeline(kea.device());

        let (storage_image, storage_image_view, allocation) = Self::create_storage_image(
            kea.device(),
            kea.presenter().format(),
            kea.presenter().size(),
            &command_pool,
        );

        let descriptor_set = Self::create_descriptor_set(
            kea.device(),
            &descriptor_set_layout,
            &tl_acceleration_structure,
            storage_image_view,
            &spheres_buffer,
        );

        let (shader_binding_tables_buffer, shader_binding_tables) =
            Self::create_shader_binding_tables(
                kea.device(),
                &pipeline,
                &kea.physical_device().ray_tracing_pipeline_properties(),
            );

        PathTracer {
            kea,
            _command_pool: command_pool,
            _tl_acceleration_structure: tl_acceleration_structure,
            _bl_acceleration_structure: bl_acceleration_structure,
            pipeline,
            pipeline_layout,
            _descriptor_set_layout: descriptor_set_layout,
            descriptor_set,
            _spheres_buffer: spheres_buffer,
            _aabbs_buffer: aabbs_buffer,
            storage_image,
            storage_image_view,
            allocation: ManuallyDrop::new(allocation),
            _shader_binding_tables_buffer: shader_binding_tables_buffer,
            shader_binding_tables,
        }
    }

    fn build_acceleration_structure(
        kea: &Kea,
    ) -> (
        AccelerationStructure,
        AccelerationStructure,
        AllocatedBuffer,
        AllocatedBuffer,
    ) {
        let spheres = [
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
        ];

        let (spheres_buffer, aabbs_buffer) = Self::create_buffers(kea.device(), &spheres);
        let geometries = [Geometry::aabbs(&aabbs_buffer)];
        let blas = AccelerationStructureDescription::new(
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            &geometries,
        );

        let build_sizes = blas.build_sizes(kea.device());
        let scratch_buffer = Buffer::new(
            kea.device().clone(),
            build_sizes.build_scratch,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("scratch", MemoryLocation::GpuOnly);

        let bl_acceleration_structure_buffer = Buffer::new(
            kea.device().clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("bl acceleration structure", MemoryLocation::GpuOnly);

        let bl_acceleration_structure = AccelerationStructure::new(
            kea.device(),
            bl_acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
        );

        let cmd = CommandPool::new(kea.device().graphics_queue())
            .allocate_buffer()
            .record(|cmd| {
                cmd.build_acceleration_structure(
                    &blas,
                    &bl_acceleration_structure,
                    &scratch_buffer,
                );
            })
            .submit()
            .wait_and_reset();

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
            kea.device().clone(),
            mem::size_of_val(&tlas_instance) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("tlas build", MemoryLocation::CpuToGpu);
        tlas_buffer.fill(&[tlas_instance]);

        let geometries = [Geometry::instances(&tlas_buffer)];
        let tlas = AccelerationStructureDescription::new(
            vk::AccelerationStructureTypeKHR::TOP_LEVEL,
            &geometries,
        );

        let build_sizes = tlas.build_sizes(kea.device());
        let scratch_buffer = Buffer::new(
            kea.device().clone(),
            build_sizes.build_scratch,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("build scratch", MemoryLocation::GpuOnly);

        let tl_acceleration_structure_buffer = Buffer::new(
            kea.device().clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("tl acceleration structure", MemoryLocation::GpuOnly);

        let tl_acceleration_structure = AccelerationStructure::new(
            kea.device(),
            tl_acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::TOP_LEVEL,
        );

        cmd.record(|cmd| {
            cmd.build_acceleration_structure(&tlas, &tl_acceleration_structure, &scratch_buffer);
        })
        .submit()
        .wait();

        (
            tl_acceleration_structure,
            bl_acceleration_structure,
            spheres_buffer,
            aabbs_buffer,
        )
    }

    fn create_buffers(
        device: &Arc<Device>,
        spheres: &[Sphere],
    ) -> (AllocatedBuffer, AllocatedBuffer) {
        let spheres_buffer = Buffer::new(
            device.clone(),
            (mem::size_of::<Sphere>() * spheres.len()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let spheres_buffer = spheres_buffer.allocate("spheres", MemoryLocation::CpuToGpu);
        info!("spheres data {:?}", spheres);
        spheres_buffer.fill(spheres);

        let aabbs: Vec<vk::AabbPositionsKHR> = spheres
            .iter()
            .map(|s: &Sphere| {
                let aabb = s.aabb();
                vk::AabbPositionsKHR {
                    min_x: aabb.min.x,
                    min_y: aabb.min.y,
                    min_z: aabb.min.z,
                    max_x: aabb.max.x,
                    max_y: aabb.max.y,
                    max_z: aabb.max.z,
                }
            })
            .collect();
        let aabbs_buffer = Buffer::new(
            device.clone(),
            (mem::size_of::<vk::AabbPositionsKHR>() * aabbs.len()) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let aabbs_buffer = aabbs_buffer.allocate("aabbs", MemoryLocation::CpuToGpu);
        aabbs_buffer.fill(&aabbs);

        (spheres_buffer, aabbs_buffer)
    }

    fn create_pipeline(device: &Arc<Device>) -> (Pipeline, PipelineLayout, DescriptorSetLayout) {
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
            DescriptorSetLayoutBinding::new(
                2,
                vk::DescriptorType::STORAGE_BUFFER,
                1,
                vk::ShaderStageFlags::INTERSECTION_KHR,
            ),
        ];

        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), &bindings);
        let pipeline_layout =
            PipelineLayout::new(device.clone(), slice::from_ref(&descriptor_set_layout));

        let shader_module = ShaderModule::new(device.clone(), "./kea_renderer_shaders");
        let generate_rays = shader_module.entry_point("generate_rays");
        let ray_miss = shader_module.entry_point("ray_miss");
        let ray_hit = shader_module.entry_point("ray_hit");
        let intersect_sphere = shader_module.entry_point("intersect_sphere");

        let shader_stages = [
            PipelineShaderStage::new(vk::ShaderStageFlags::RAYGEN_KHR, &generate_rays),
            PipelineShaderStage::new(vk::ShaderStageFlags::MISS_KHR, &ray_miss),
            PipelineShaderStage::new(vk::ShaderStageFlags::CLOSEST_HIT_KHR, &ray_hit),
            PipelineShaderStage::new(vk::ShaderStageFlags::INTERSECTION_KHR, &intersect_sphere),
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

        (pipeline, pipeline_layout, descriptor_set_layout)
    }

    fn create_storage_image(
        device: &Device,
        format: vk::Format,
        size: (u32, u32),
        command_pool: &Arc<CommandPool>,
    ) -> (vk::Image, vk::ImageView, Allocation) {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: size.0,
                height: size.1,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe { device.raw().create_image(&image_create_info, None) }.unwrap();
        let requirements = unsafe { device.raw().get_image_memory_requirements(image) };

        let allocation = device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: "rt image output",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: true,
            })
            .unwrap();

        unsafe {
            device
                .raw()
                .bind_image_memory(image, allocation.memory(), allocation.offset())
        }
        .unwrap();

        let view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image);

        let image_view = unsafe { device.raw().create_image_view(&view_info, None) }.unwrap();

        command_pool
            .allocate_buffer()
            .record(|cmd| {
                cmd.transition_image_layout(
                    image,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::GENERAL,
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::empty(),
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )
            })
            .submit()
            .wait();

        (image, image_view, allocation)
    }

    fn create_descriptor_set(
        device: &Arc<Device>,
        layout: &DescriptorSetLayout,
        tl_acceleration_structure: &AccelerationStructure,
        storage_image_view: vk::ImageView,
        spheres_buffer: &AllocatedBuffer,
    ) -> DescriptorSet {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool = DescriptorPool::new(device.clone(), 1, &pool_sizes);
        let descriptor_sets = descriptor_pool.allocate_descriptor_sets(slice::from_ref(layout));
        let descriptor_set = descriptor_sets.into_iter().nth(0).unwrap();

        let raw_as = unsafe { tl_acceleration_structure.raw() };
        let accel_slice = std::slice::from_ref(&raw_as);
        let mut write_set_as = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(accel_slice);
        let mut as_write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .dst_set(unsafe { descriptor_set.raw() })
            .dst_binding(0)
            .push_next(&mut write_set_as)
            .build();
        as_write_set.descriptor_count = 1;

        let desc_img_info = vk::DescriptorImageInfo::builder()
            .image_view(storage_image_view)
            .image_layout(vk::ImageLayout::GENERAL);

        let img_write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .dst_set(unsafe { descriptor_set.raw() })
            .dst_binding(1)
            .image_info(slice::from_ref(&desc_img_info))
            .build();

        let sphere_buffer_info = vk::DescriptorBufferInfo {
            buffer: unsafe { spheres_buffer.buffer().raw() },
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let spheres_write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_set(unsafe { descriptor_set.raw() })
            .dst_binding(2)
            .buffer_info(slice::from_ref(&sphere_buffer_info))
            .build();

        let write_sets = [as_write_set, img_write_set, spheres_write_set];

        unsafe { device.raw().update_descriptor_sets(&write_sets, &[]) };
        descriptor_set
    }

    fn create_shader_binding_tables(
        device: &Arc<Device>,
        pipeline: &Pipeline,
        rt_pipeline_props: &vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
    ) -> (AllocatedBuffer, RayTracingShaderBindingTables) {
        let handle_size = rt_pipeline_props.shader_group_handle_size;
        let handle_alignment =
            Self::aligned_size(handle_size, rt_pipeline_props.shader_group_handle_alignment);
        let aligned_handle_size = Self::aligned_size(handle_size, handle_alignment);
        let handle_pad = aligned_handle_size - handle_size;

        let group_alignment = rt_pipeline_props.shader_group_base_alignment;

        let group_count = 3;
        //
        let data_size = group_count * handle_size;

        let group_handles = unsafe {
            device
                .ext()
                .ray_tracing_pipeline()
                .get_ray_tracing_shader_group_handles(
                    pipeline.raw(),
                    0,
                    group_count,
                    data_size as _,
                )
        }
        .unwrap();

        let raygen_count = 1;
        let raygen_region_size =
            Self::aligned_size(raygen_count * aligned_handle_size, group_alignment);

        let miss_count = 1;
        let miss_region_size =
            Self::aligned_size(miss_count * aligned_handle_size, group_alignment);

        let hit_count = 1;
        let hit_region_size = Self::aligned_size(hit_count * aligned_handle_size, group_alignment);

        let buffer_size = raygen_region_size + miss_region_size + hit_region_size;
        let mut aligned_handles = Vec::<u8>::with_capacity(buffer_size as _);

        let groups_shader_count = [raygen_count, miss_count, hit_count];
        let mut offset = 0;
        // for each groups
        for group_shader_count in groups_shader_count {
            let group_size = group_shader_count * aligned_handle_size;
            let aligned_group_size =
                Self::aligned_size(group_size, rt_pipeline_props.shader_group_base_alignment);
            let group_pad = aligned_group_size - group_size;

            // for each handle
            for _ in 0..group_shader_count {
                //copy handle
                for _ in 0..handle_size as usize {
                    aligned_handles.push(group_handles[offset]);
                    offset += 1;
                }

                // pad handle to alignment
                for _ in 0..handle_pad {
                    aligned_handles.push(0);
                }
            }

            // pad group to alignment
            for _ in 0..group_pad {
                aligned_handles.push(0);
            }
        }

        let binding_table_buffer = Buffer::new(
            device.clone(),
            buffer_size as _,
            vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate("rt shader binding table", MemoryLocation::CpuToGpu);

        info!("unaligned shader handles {:?}", group_handles);
        info!("aligned shader handles {:?}", aligned_handles);

        binding_table_buffer.fill(&aligned_handles);

        let buffer_address = binding_table_buffer.device_address();

        let tables = RayTracingShaderBindingTables {
            raygen: ShaderBindingTable::new(
                buffer_address,
                raygen_region_size as _,
                raygen_region_size as _,
            ),

            miss: ShaderBindingTable::new(
                buffer_address + raygen_region_size as u64,
                miss_region_size as _,
                aligned_handle_size as _,
            ),

            hit: ShaderBindingTable::new(
                buffer_address + raygen_region_size as u64 + miss_region_size as u64,
                hit_region_size as _,
                aligned_handle_size as _,
            ),

            callable: ShaderBindingTable::empty(),
        };

        (binding_table_buffer, tables)
    }

    fn aligned_size(size: u32, alignment: u32) -> u32 {
        // from nvh::align_up
        (size + (alignment - 1)) & !(alignment - 1)
    }

    pub fn draw(&self) {
        self.kea.presenter().draw(|cmd, swapchain_image_view| {
            cmd.bind_pipeline(vk::PipelineBindPoint::RAY_TRACING_KHR, &self.pipeline);
            cmd.bind_descriptor_sets(
                vk::PipelineBindPoint::RAY_TRACING_KHR,
                &self.pipeline_layout,
                slice::from_ref(&self.descriptor_set),
            );

            cmd.trace_rays(
                &self.shader_binding_tables,
                (
                    self.kea.presenter().size().0,
                    self.kea.presenter().size().1,
                    1,
                ),
            );

            cmd.transition_image_layout(
                swapchain_image_view.image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            );

            cmd.transition_image_layout(
                self.storage_image,
                vk::ImageLayout::GENERAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_READ,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            );

            let copy_region = vk::ImageCopy::builder()
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_array_layer: 0,
                    mip_level: 0,
                    layer_count: 1,
                })
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_array_layer: 0,
                    mip_level: 0,
                    layer_count: 1,
                })
                .extent(vk::Extent3D {
                    width: self.kea.presenter().size().0,
                    height: self.kea.presenter().size().1,
                    depth: 1,
                })
                .build();

            cmd.copy_image(self.storage_image, swapchain_image_view.image, &copy_region);

            cmd.transition_image_layout(
                swapchain_image_view.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::COLOR_ATTACHMENT_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );

            cmd.transition_image_layout(
                self.storage_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::GENERAL,
                vk::AccessFlags::TRANSFER_READ,
                vk::AccessFlags::empty(),
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TOP_OF_PIPE,
            );
        })
    }
}

impl Drop for PathTracer {
    fn drop(&mut self) {
        unsafe {
            self.kea.device().wait_until_idle();
            self.kea
                .device()
                .raw()
                .destroy_image_view(self.storage_image_view, None);
            self.kea
                .device()
                .raw()
                .destroy_image(self.storage_image, None);
            self.kea
                .device()
                .allocator()
                .lock()
                .unwrap()
                .free(ManuallyDrop::take(&mut self.allocation))
                .unwrap();
        }
    }
}
