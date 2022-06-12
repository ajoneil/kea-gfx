use std::sync::Arc;

use ash::vk;

use super::{
    command::{CommandBuffer, CommandPool},
    Device, RasterizationPipeline, Swapchain,
};

pub struct Rasterizer {
    swapchain: Swapchain,
    pipeline: RasterizationPipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_buffer: CommandBuffer,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl Rasterizer {
    pub fn new(swapchain: Swapchain) -> Rasterizer {
        let pipeline = RasterizationPipeline::new(&swapchain.device, swapchain.format);
        let framebuffers = Self::create_framebuffers(
            &swapchain.device,
            pipeline.render_pass,
            &swapchain.image_views,
        );

        let command_buffer = Arc::new(CommandPool::new(swapchain.device.clone())).allocate_buffer();

        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            Self::create_sync_objects(&swapchain.device);

        Rasterizer {
            swapchain,
            pipeline,
            framebuffers,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
    ) -> Vec<vk::Framebuffer> {
        image_views
            .iter()
            .map(|image_view| {
                let attachments = [*image_view];
                let framebuffer = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(1920)
                    .height(1080)
                    .layers(1);

                unsafe { device.device.create_framebuffer(&framebuffer, None) }.unwrap()
            })
            .collect()
    }

    fn create_sync_objects(device: &Device) -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
        let image_available_semaphore = unsafe {
            device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();
        let render_finished_semaphore = unsafe {
            device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();
        let in_flight_fence = unsafe {
            device.device.create_fence(
                &vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
        }
        .unwrap();

        (
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        )
    }

    fn record_command_buffer(&self, image_index: u32) {
        let begin_command_buffer = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.swapchain
                .device
                .device
                .begin_command_buffer(self.command_buffer.buffer, &begin_command_buffer)
        }
        .unwrap();

        let begin_render_pass = vk::RenderPassBeginInfo::builder()
            .render_pass(self.pipeline.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1920,
                    height: 1080,
                },
            })
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }]);

        unsafe {
            self.swapchain.device.device.cmd_begin_render_pass(
                self.command_buffer.buffer,
                &begin_render_pass,
                vk::SubpassContents::INLINE,
            );

            self.swapchain.device.device.cmd_bind_pipeline(
                self.command_buffer.buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.swapchain
                .device
                .device
                .cmd_draw(self.command_buffer.buffer, 3, 1, 0, 0);
            self.swapchain
                .device
                .device
                .cmd_end_render_pass(self.command_buffer.buffer);
            self.swapchain
                .device
                .device
                .end_command_buffer(self.command_buffer.buffer)
        }
        .unwrap();
    }

    pub fn draw(&self) {
        unsafe {
            self.swapchain
                .device
                .device
                .wait_for_fences(&[self.in_flight_fence], true, u64::MAX)
                .unwrap();
            self.swapchain
                .device
                .device
                .reset_fences(&[self.in_flight_fence])
                .unwrap();

            let (image_index, _) = self
                .swapchain
                .device
                .ext
                .swapchain
                .acquire_next_image(
                    self.swapchain.swapchain,
                    u64::MAX,
                    self.image_available_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();

            self.swapchain
                .device
                .device
                .reset_command_buffer(
                    self.command_buffer.buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();

            self.record_command_buffer(image_index);

            let submits = [vk::SubmitInfo::builder()
                .wait_semaphores(&[self.image_available_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.command_buffer.buffer])
                .signal_semaphores(&[self.render_finished_semaphore])
                .build()];

            self.swapchain
                .device
                .device
                .queue_submit(self.swapchain.device.queue, &submits, self.in_flight_fence)
                .unwrap();

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[self.render_finished_semaphore])
                .swapchains(&[self.swapchain.swapchain])
                .image_indices(&[image_index])
                .build();

            self.swapchain
                .device
                .ext
                .swapchain
                .queue_present(self.swapchain.device.queue, &present)
                .unwrap();
        }
    }
}

impl Drop for Rasterizer {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.device.device.device_wait_idle().unwrap();

            self.swapchain
                .device
                .device
                .destroy_semaphore(self.image_available_semaphore, None);
            self.swapchain
                .device
                .device
                .destroy_semaphore(self.render_finished_semaphore, None);
            self.swapchain
                .device
                .device
                .destroy_fence(self.in_flight_fence, None);

            for &framebuffer in self.framebuffers.iter() {
                self.swapchain
                    .device
                    .device
                    .destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
