use std::sync::Arc;

use ash::vk;

use super::{
    command::{CommandBuffer, CommandPool},
    sync::{Fence, Semaphore},
    Device, RasterizationPipeline, Swapchain,
};

struct Semaphores {
    image_available: Semaphore,
    render_finished: Semaphore,
}

pub struct Rasterizer {
    swapchain: Swapchain,
    pipeline: RasterizationPipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_buffer: CommandBuffer,
    semaphores: Semaphores,
    in_flight_fence: Fence,
}

impl Rasterizer {
    pub fn new(swapchain: Swapchain) -> Rasterizer {
        let pipeline = RasterizationPipeline::new(&swapchain.device, swapchain.format);
        let framebuffers = Self::create_framebuffers(
            &swapchain.device,
            pipeline.render_pass,
            &swapchain.image_views,
        );

        let semaphores = Semaphores {
            image_available: Semaphore::new(swapchain.device.clone()),
            render_finished: Semaphore::new(swapchain.device.clone()),
        };
        let in_flight_fence = Fence::new(swapchain.device.clone(), true);

        let command_buffer = Arc::new(CommandPool::new(swapchain.device.clone())).allocate_buffer();

        Rasterizer {
            swapchain,
            pipeline,
            framebuffers,
            command_buffer,
            semaphores,
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

                unsafe { device.vk().create_framebuffer(&framebuffer, None) }.unwrap()
            })
            .collect()
    }

    fn record_command_buffer(&self, image_index: u32) {
        self.command_buffer.record(true, |cmd| {
            cmd.render_pass(
                self.pipeline.render_pass,
                self.framebuffers[image_index as usize],
                || {
                    cmd.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);
                    cmd.draw(3, 1, 0, 0);
                },
            )
        });
    }

    pub fn draw(&self) {
        self.in_flight_fence.wait_and_reset();

        let image_index = self
            .swapchain
            .acquire_next_image(&self.semaphores.image_available);

        self.record_command_buffer(image_index);

        unsafe {
            let submits = [vk::SubmitInfo::builder()
                .wait_semaphores(&[self.semaphores.image_available.vk()])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.command_buffer.buffer])
                .signal_semaphores(&[self.semaphores.render_finished.vk()])
                .build()];

            self.swapchain
                .device
                .vk()
                .queue_submit(
                    self.swapchain.device.queue,
                    &submits,
                    self.in_flight_fence.vk(),
                )
                .unwrap();

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[self.semaphores.render_finished.vk()])
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
            self.swapchain.device.vk().device_wait_idle().unwrap();

            for &framebuffer in self.framebuffers.iter() {
                self.swapchain
                    .device
                    .vk()
                    .destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
