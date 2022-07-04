use ash::vk;
use glam::vec3;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    commands::CommandBuffer,
    core::{
        pipeline::{
            Pipeline, PipelineDescription, PipelineLayout, PipelineShaderStage,
            RayTracingPipelineDescription,
        },
        shaders::ShaderModule,
    },
    descriptors::{DescriptorPool, DescriptorSet, DescriptorSetLayout},
    device::Device,
    ray_tracing::{
        scenes::{Geometry, GeometryInstance, Scene},
        RayTracingShaderBindingTables, ShaderBindingTable,
    },
    slots::SlotLayout,
    storage::{
        buffers::{AlignedBuffer, Buffer, TransferBuffer},
        images::{Image, ImageView},
        memory,
    },
    Kea,
};
use kea_gpu_shaderlib::Aabb;
use kea_renderer_shaders::Sphere;
use log::info;
use std::{slice, sync::Arc};

pub struct PathTracer {
    kea: Kea,
    _scene: Scene,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    _descriptor_set_layout: DescriptorSetLayout,
    descriptor_set: DescriptorSet,
    storage_image: ImageView,
    _shader_binding_tables_buffer: AlignedBuffer,
    shader_binding_tables: RayTracingShaderBindingTables,
}

impl PathTracer {
    pub fn new(kea: Kea) -> PathTracer {
        let scene = Self::build_scene(&kea);

        let storage_image = Self::create_storage_image(
            kea.device(),
            kea.presenter().format(),
            kea.presenter().size(),
        );

        let (pipeline, pipeline_layout, descriptor_set_layout) =
            Self::create_pipeline(kea.device());

        let descriptor_set = Self::create_descriptor_set(
            kea.device(),
            &descriptor_set_layout,
            &scene,
            &storage_image,
            &scene
                .instances()
                .iter()
                .nth(0)
                .unwrap()
                .geometry()
                .additional_data(),
        );

        let (shader_binding_tables_buffer, shader_binding_tables) =
            Self::create_shader_binding_tables(
                kea.device(),
                &pipeline,
                &kea.physical_device().ray_tracing_pipeline_properties(),
            );

        PathTracer {
            kea,
            _scene: scene,
            pipeline,
            pipeline_layout,
            _descriptor_set_layout: descriptor_set_layout,
            descriptor_set,
            storage_image,
            _shader_binding_tables_buffer: shader_binding_tables_buffer,
            shader_binding_tables,
        }
    }

    fn build_scene(kea: &Kea) -> Scene {
        let spheres = [
            Sphere {
                position: vec3(0.0, 0.0, 1.5),
                radius: 0.5,
            },
            Sphere {
                position: vec3(1.0, 1.0, 1.5),
                radius: 1.5,
            },
            Sphere {
                position: vec3(-0.5, 0.0, -1.5),
                radius: 2.5,
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
        let mut geometry = Geometry::new(
            "spheres".to_string(),
            aabbs_buffer,
            Arc::new(spheres_buffer),
        );
        geometry.build();
        let geometry_instance = GeometryInstance::new(Arc::new(geometry));

        let mut scene = Scene::new(kea.device().clone(), "test scene".to_string());
        scene.add_instance(geometry_instance);
        scene.build();

        scene
    }

    fn create_buffers(device: &Arc<Device>, spheres: &[Sphere]) -> (Buffer, Buffer) {
        let spheres_buffer = Buffer::new_from_data(
            device.clone(),
            spheres,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "spheres".to_string(),
            MemoryLocation::GpuOnly,
        );
        info!("spheres data {:?}", spheres);

        let aabbs: Vec<Aabb> = spheres.iter().map(|s: &Sphere| s.aabb()).collect();
        log::debug!("Aabbs: {:?}", aabbs);
        let aabbs_buffer = Buffer::new_from_data(
            device.clone(),
            &aabbs,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            "aabbs".to_string(),
            MemoryLocation::GpuOnly,
        );

        (spheres_buffer, aabbs_buffer)
    }

    fn create_pipeline(device: &Arc<Device>) -> (Pipeline, PipelineLayout, DescriptorSetLayout) {
        let slot_layout = SlotLayout::new(kea_renderer_shaders::SLOTS.to_vec());
        let bindings = slot_layout.bindings();

        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), &bindings);
        let pipeline_layout =
            PipelineLayout::new(device.clone(), slice::from_ref(&descriptor_set_layout));

        let shader_modules =
            ShaderModule::new_multimodule(&device.clone(), "./kea_renderer_shaders");
        // let shader_module = ShaderModule::new(device.clone(), "./kea_renderer_shaders");
        let generate_rays = shader_modules["generate_rays"].entry_point("generate_rays");
        let ray_miss = shader_modules["ray_miss"].entry_point("ray_miss");
        let ray_hit = shader_modules["ray_hit"].entry_point("ray_hit");
        let intersect_sphere = shader_modules["intersect_sphere"].entry_point("intersect_sphere");

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
        device: &Arc<Device>,
        format: vk::Format,
        size: (u32, u32),
    ) -> ImageView {
        let image = Image::new(
            device.clone(),
            "rt image output".to_string(),
            size,
            format,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
            MemoryLocation::GpuOnly,
        );

        let image_view = ImageView::new(Arc::new(image));

        CommandBuffer::now(
            device,
            "Set initial rt output image layout".to_string(),
            |cmd| {
                cmd.transition_image_layout(
                    &image_view.image(),
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::GENERAL,
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::empty(),
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )
            },
        );

        image_view
    }

    fn create_descriptor_set(
        device: &Arc<Device>,
        layout: &DescriptorSetLayout,
        scene: &Scene,
        storage_image: &ImageView,
        spheres_buffer: &Buffer,
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

        let raw_as = unsafe { scene.acceleration_structure().raw() };
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
            .image_view(unsafe { storage_image.raw() })
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
    ) -> (AlignedBuffer, RayTracingShaderBindingTables) {
        let handle_size = rt_pipeline_props.shader_group_handle_size;
        let handle_alignment =
            memory::align(handle_size, rt_pipeline_props.shader_group_handle_alignment);
        let aligned_handle_size = memory::align(handle_size, handle_alignment);
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
        let raygen_region_size = memory::align(raygen_count * aligned_handle_size, group_alignment);

        let miss_count = 1;
        let miss_region_size = memory::align(miss_count * aligned_handle_size, group_alignment);

        let hit_count = 1;
        let hit_region_size = memory::align(hit_count * aligned_handle_size, group_alignment);

        let buffer_size = raygen_region_size + miss_region_size + hit_region_size;
        let mut aligned_handles = Vec::<u8>::with_capacity(buffer_size as _);

        let groups_shader_count = [raygen_count, miss_count, hit_count];
        let mut offset = 0;
        // for each groups
        for group_shader_count in groups_shader_count {
            let group_size = group_shader_count * aligned_handle_size;
            let aligned_group_size =
                memory::align(group_size, rt_pipeline_props.shader_group_base_alignment);
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

        let mut binding_table_buffer = TransferBuffer::new(
            device.clone(),
            buffer_size as _,
            vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR,
            "rt shader binding table".to_string(),
        );

        info!("unaligned shader handles {:?}", group_handles);
        info!("aligned shader handles {:?}", aligned_handles);

        binding_table_buffer.cpu_buffer().fill(&aligned_handles);
        let binding_table_buffer =
            binding_table_buffer.transfer_to_gpu_with_alignment(group_alignment);

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

    pub fn draw(&self) {
        self.kea.presenter().draw(|cmd, swapchain_image| {
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
                &swapchain_image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            );

            cmd.transition_image_layout(
                &self.storage_image.image(),
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

            cmd.copy_image(
                &self.storage_image.image(),
                &swapchain_image.image(),
                &copy_region,
            );

            cmd.transition_image_layout(
                swapchain_image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::COLOR_ATTACHMENT_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );

            cmd.transition_image_layout(
                &self.storage_image.image(),
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
