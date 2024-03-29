use crate::scenes::{self, Scene};
use ash::vk;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    commands::{CommandBuffer, CommandPool, RecordedCommandBuffer},
    descriptors::DescriptorSetLayout,
    device::Device,
    pipelines::PipelineLayout,
    ray_tracing::RayTracingPipeline,
    shaders::ShaderGroups,
    slots::{SlotBindings, SlotLayout},
    storage::images::{Image, ImageView},
    Kea,
};
use kea_renderer_shaders::{path_tracer::entrypoints::PushConstants, SlotId};
use std::{
    cell::{Cell, RefCell},
    slice,
    sync::Arc,
};

pub struct PathTracer {
    kea: Kea,
    _scene: Scene,
    pipeline: RayTracingPipeline<SlotId>,
    slot_bindings: SlotBindings<SlotId>,
    storage_image: Arc<ImageView>,
    _light_image: Arc<ImageView>,
    commands: RefCell<Vec<RecordedCommandBuffer>>,
    iteration: Cell<u64>,
}

impl PathTracer {
    pub fn new(kea: Kea) -> PathTracer {
        let pipeline = Self::create_pipeline(kea.device());
        let mut slot_bindings = SlotBindings::new(kea.device().clone(), &pipeline);

        let storage_image = Self::create_storage_image(
            kea.device(),
            kea.presenter().format(),
            kea.presenter().size(),
        );
        slot_bindings.bind_image(SlotId::OutputImage, storage_image.clone());

        let light_image = Self::create_storage_image(
            kea.device(),
            vk::Format::R32G32B32A32_SFLOAT,
            kea.presenter().size(),
        );
        slot_bindings.bind_image(SlotId::LightImage, light_image.clone());

        let scene = scenes::examples::cornell_box(kea.device().clone());
        scene.bind_data(&mut slot_bindings);

        PathTracer {
            kea,
            _scene: scene,
            pipeline,
            slot_bindings,
            storage_image,
            _light_image: light_image,
            commands: RefCell::new(vec![]),
            iteration: Cell::new(0),
        }
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
    ) -> Arc<ImageView> {
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
                    vk::AccessFlags2::empty(),
                    vk::AccessFlags2::empty(),
                    vk::PipelineStageFlags2::TOP_OF_PIPE,
                    vk::PipelineStageFlags2::TOP_OF_PIPE,
                )
            },
        );

        Arc::new(image_view)
    }

    pub fn draw(&self) {
        let (swapchain_index, swapchain_image) = self.kea.presenter().get_swapchain_image();

        // if self.commands.borrow().len() == swapchain_index as usize {
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
                    slice::from_ref(&self.slot_bindings.descriptor_set()),
                );
                unsafe {
                    let constants = PushConstants {
                        iteration: self.iteration.get(),
                    };
                    self.iteration.set(self.iteration.get() + 1);
                    let (_, constants, _) = slice::from_ref(&constants).align_to::<u8>();
                    self.kea.device().raw().cmd_push_constants(
                        cmd.buffer().raw(),
                        self.pipeline.layout().raw(),
                        vk::ShaderStageFlags::RAYGEN_KHR,
                        0,
                        constants,
                    );
                }

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
                    vk::AccessFlags2::empty(),
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::TOP_OF_PIPE,
                    vk::PipelineStageFlags2::TRANSFER,
                );

                cmd.transition_image_layout(
                    &self.storage_image.image(),
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::AccessFlags2::empty(),
                    vk::AccessFlags2::TRANSFER_READ,
                    vk::PipelineStageFlags2::TOP_OF_PIPE,
                    vk::PipelineStageFlags2::TRANSFER,
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
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                );

                cmd.transition_image_layout(
                    &self.storage_image.image(),
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::GENERAL,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::AccessFlags2::empty(),
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::TOP_OF_PIPE,
                );
            });

        //     self.commands.borrow_mut().push(cmd);
        // }

        // let command = &self.commands.borrow()[swapchain_index as usize];

        self.kea
            .presenter()
            .draw(swapchain_index, slice::from_ref(&cmd));
    }
}

impl Drop for PathTracer {
    fn drop(&mut self) {
        self.commands.take().into_iter().for_each(|c| unsafe {
            c.consume();
        });
    }
}
