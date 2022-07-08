use ash::vk;
use glam::vec3;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    commands::{CommandBuffer, CommandPool, RecordedCommandBuffer},
    descriptors::{DescriptorPool, DescriptorSet, DescriptorSetLayout},
    device::Device,
    pipelines::PipelineLayout,
    ray_tracing::{
        scenes::{Geometry, GeometryInstance, GeometryType, Scene},
        RayTracingPipeline,
    },
    shaders::ShaderGroups,
    slots::SlotLayout,
    storage::{
        buffers::Buffer,
        images::{Image, ImageView},
    },
    Kea,
};
use kea_gpu_shaderlib::Aabb;
use kea_renderer_shaders::{spheres::Sphere, SlotId};
use log::info;
use std::{cell::RefCell, slice, sync::Arc};

pub struct PathTracer {
    kea: Kea,
    _scene: Scene,
    pipeline: RayTracingPipeline<SlotId>,
    descriptor_set: DescriptorSet,
    storage_image: ImageView,
    commands: RefCell<Vec<RecordedCommandBuffer>>,
}

impl PathTracer {
    pub fn new(kea: Kea) -> PathTracer {
        let scene = Self::build_scene(&kea);

        let storage_image = Self::create_storage_image(
            kea.device(),
            kea.presenter().format(),
            kea.presenter().size(),
        );

        let pipeline = Self::create_pipeline(kea.device());

        let descriptor_set = Self::create_descriptor_set(
            kea.device(),
            &pipeline,
            &scene,
            &storage_image,
            scene
                .instances()
                .iter()
                .nth(0)
                .unwrap()
                .geometry()
                .additional_data()
                .as_ref()
                .unwrap(),
        );

        PathTracer {
            kea,
            _scene: scene,
            pipeline,
            descriptor_set,
            storage_image,
            commands: RefCell::new(vec![]),
        }
    }

    fn build_scene(kea: &Kea) -> Scene {
        let mut scene = Scene::new(kea.device().clone(), "test scene".to_string());

        let spheres = [
            Sphere::new(vec3(0.0, -100.4, -1.5), 100.0),
            Sphere::new(vec3(0.0, 0.4, -1.5), 0.4),
            Sphere::new(vec3(-1.0, 0.4, -1.7), 0.4),
            Sphere::new(vec3(1.0, 0.4, -1.6), 0.4),
            // Sphere::new(vec3(-0.5, 0.0, -1.5), 0.5),
            // Sphere::new(vec3(0.0, 0.0, -1.5), 0.5),
            // Sphere::new(vec3(0.0, 0.0, -1.5), 0.5),
            // Sphere::new(vec3(0.0, 0.0, -1.5), 0.5),
        ];

        let (spheres_buffer, aabbs_buffer) = Self::create_buffers(kea.device(), &spheres);
        let mut geometry = Geometry::new(
            kea.device().clone(),
            "spheres".to_string(),
            GeometryType::Aabbs(aabbs_buffer),
            Some(Arc::new(spheres_buffer)),
        );
        geometry.build();
        let geometry_instance = GeometryInstance::new(Arc::new(geometry), 1);
        scene.add_instance(geometry_instance);

        // let vertices = [
        //     vec3(-0.2, -0.2, -0.5),
        //     vec3(0.2, -0.2, -0.5),
        //     vec3(0.0, 0.2, -0.5),
        // ];

        // let vertex_buffer = Buffer::new_from_data(
        //     kea.device().clone(),
        //     &vertices,
        //     vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        //     "vertices".to_string(),
        //     MemoryLocation::GpuOnly,
        //     None,
        // );

        // const INDICES: [u16; 3] = [0, 1, 2];

        // let index_buffer = Buffer::new_from_data(
        //     kea.device().clone(),
        //     &INDICES,
        //     vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        //     "indices".to_string(),
        //     MemoryLocation::GpuOnly,
        //     None,
        // );

        // let mut geometry = Geometry::new(
        //     kea.device().clone(),
        //     "triangles".to_string(),
        //     GeometryType::Triangles {
        //         vertices: vertex_buffer,
        //         indices: index_buffer,
        //     },
        //     None,
        // );
        // geometry.build();

        // let geometry_instance = GeometryInstance::new(Arc::new(geometry), 0);
        // scene.add_instance(geometry_instance);

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
            None,
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
            None,
        );

        (spheres_buffer, aabbs_buffer)
    }

    fn create_pipeline(device: &Arc<Device>) -> RayTracingPipeline<SlotId> {
        let slot_layout = SlotLayout::new(kea_renderer_shaders::SLOTS.to_vec());
        let bindings = slot_layout.bindings();

        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), &bindings);
        let pipeline_layout = PipelineLayout::new(device.clone(), descriptor_set_layout);

        let shader_groups = ShaderGroups::new(kea_renderer_shaders::SHADERS.to_vec());
        let pipeline_shaders = shader_groups.build(device.clone(), "./kea_renderer_shaders");
        let pipeline = RayTracingPipeline::<SlotId>::new(
            device.clone(),
            shader_groups,
            pipeline_shaders,
            pipeline_layout,
            slot_layout,
        );

        pipeline
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
        pipeline: &RayTracingPipeline<SlotId>,
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
        let descriptor_sets = descriptor_pool
            .allocate_descriptor_sets(slice::from_ref(pipeline.layout().descriptor_set_layout()));
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
            .image_info(slice::from_ref(&desc_img_info));

        let sphere_buffer_info = vk::DescriptorBufferInfo {
            buffer: unsafe { spheres_buffer.buffer().raw() },
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let spheres_write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_set(unsafe { descriptor_set.raw() })
            .dst_binding(2)
            .buffer_info(slice::from_ref(&sphere_buffer_info));

        let write_sets = [as_write_set, *img_write_set, *spheres_write_set];

        unsafe { device.raw().update_descriptor_sets(&write_sets, &[]) };
        descriptor_set
    }

    pub fn draw(&self) {
        let (swapchain_index, swapchain_image) = self.kea.presenter().get_swapchain_image();

        if self.commands.borrow().len() == swapchain_index as usize {
            let cmd = CommandPool::new(self.kea.device().graphics_queue())
                .allocate_buffer("trace rays".to_string())
                .record(|cmd| {
                    cmd.bind_pipeline(
                        vk::PipelineBindPoint::RAY_TRACING_KHR,
                        &self.pipeline.pipeline(),
                    );
                    cmd.bind_descriptor_sets(
                        vk::PipelineBindPoint::RAY_TRACING_KHR,
                        &self.pipeline.layout(),
                        slice::from_ref(&self.descriptor_set),
                    );

                    cmd.trace_rays(
                        self.pipeline.shader_binding_tables(),
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
                });

            self.commands.borrow_mut().push(cmd);
        }

        let command = &self.commands.borrow()[swapchain_index as usize];

        self.kea
            .presenter()
            .draw(swapchain_index, slice::from_ref(command));
    }
}

impl Drop for PathTracer {
    fn drop(&mut self) {
        self.commands.take().into_iter().for_each(|c| unsafe {
            c.consume();
        });
    }
}
