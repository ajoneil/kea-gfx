use crate::scenes::{self, Scene};

mod shader_modules {
    include!(concat!(env!("OUT_DIR"), "/shader_modules.rs"));
}
use ash::vk;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    commands::{CommandBuffer, CommandPool},
    descriptors::DescriptorSetLayout,
    device::Device,
    pipelines::PipelineLayout,
    presentation::FRAMES_IN_FLIGHT,
    ray_tracing::RayTracingPipeline,
    shaders::ShaderGroups,
    slots::{SlotBindings, SlotLayout},
    storage::images::{Image, ImageView},
    Kea,
};
use kea_renderer_shaders::{path_tracer::entrypoints::PushConstants, SlotId};
use std::{cell::RefCell, slice, sync::Arc};

struct FrameSlot {
    pool: Arc<CommandPool>,
    buffer: Option<CommandBuffer>,
}

pub struct PathTracer {
    kea: Kea,
    _scene: Scene,
    pipeline: RayTracingPipeline<SlotId>,
    slot_bindings: SlotBindings<SlotId>,
    storage_image: Arc<ImageView>,
    light_image: Arc<ImageView>,
    frame_slots: RefCell<Vec<FrameSlot>>,
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

        let frame_slots = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                let pool = CommandPool::new(kea.device().graphics_queue());
                let buffer = pool.allocate_buffer(format!("trace rays frame {}", i));
                FrameSlot {
                    pool,
                    buffer: Some(buffer),
                }
            })
            .collect();

        PathTracer {
            kea,
            _scene: scene,
            pipeline,
            slot_bindings,
            storage_image,
            light_image,
            frame_slots: RefCell::new(frame_slots),
        }
    }

    fn create_pipeline(device: &Arc<Device>) -> RayTracingPipeline<SlotId> {
        let slot_layout = SlotLayout::new(kea_renderer_shaders::SLOTS.to_vec());
        let bindings = slot_layout.bindings();

        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), &bindings);
        let pipeline_layout = PipelineLayout::new(device.clone(), descriptor_set_layout);

        let shader_groups = ShaderGroups::new(kea_renderer_shaders::SHADERS.to_vec());
        let pipeline_shaders = shader_groups.build(device.clone(), shader_modules::SHADER_MODULES);
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
                    vk::AccessFlags2::NONE,
                    vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE,
                    vk::PipelineStageFlags2::NONE,
                    vk::PipelineStageFlags2::ALL_COMMANDS,
                )
            },
        );

        Arc::new(image_view)
    }

    pub fn draw(&self) {
        let (swapchain_index, swapchain_image) = self.kea.presenter().get_swapchain_image();
        let frame = self.kea.presenter().frame_index();
        let slot_index = (frame % FRAMES_IN_FLIGHT) as usize;

        let mut slots = self.frame_slots.borrow_mut();
        let slot = &mut slots[slot_index];
        // The presenter's timeline wait at the start of the frame guarantees
        // the GPU has finished any prior use of this slot's command buffer.
        slot.pool.reset();
        let buffer = slot.buffer.take().unwrap();

        let cmd = buffer.record(|cmd| {
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
                    let constants = PushConstants { iteration: frame };
                    let (_, constants, _) = slice::from_ref(&constants).align_to::<u8>();
                    self.kea.device().raw().cmd_push_constants(
                        cmd.buffer().raw(),
                        self.pipeline.layout().raw(),
                        vk::ShaderStageFlags::RAYGEN_KHR,
                        0,
                        constants,
                    );
                }

                // light_image is read-modify-written by trace_rays each frame
                // (running-average accumulator). With FRAMES_IN_FLIGHT > 1 there
                // is no implicit ordering between consecutive frames' trace_rays,
                // so make the read of frame N+1 wait for the write of frame N.
                cmd.transition_image_layout(
                    &self.light_image.image(),
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::GENERAL,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR,
                    vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR,
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
                    vk::AccessFlags2::NONE,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::TRANSFER,
                );

                cmd.transition_image_layout(
                    &self.storage_image.image(),
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    vk::AccessFlags2::TRANSFER_READ,
                    vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR,
                    vk::PipelineStageFlags2::TRANSFER,
                );

                let copy_region = vk::ImageCopy::default()
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
                    });

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
                    vk::AccessFlags2::NONE,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::NONE,
                );

                cmd.transition_image_layout(
                    &self.storage_image.image(),
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::GENERAL,
                    vk::AccessFlags2::TRANSFER_READ,
                    vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR,
                );
            });

        self.kea
            .presenter()
            .draw(swapchain_index, slice::from_ref(&cmd));

        slot.buffer = Some(unsafe { cmd.consume() });
    }
}
