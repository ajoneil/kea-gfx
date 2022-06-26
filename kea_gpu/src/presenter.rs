use crate::core::{
    command::{CommandBuffer, CommandBufferRecorder, CommandPool},
    device::Device,
    surface::Surface,
    swapchain::{Swapchain, SwapchainImageView},
    sync::{Fence, Semaphore},
};
use ash::vk;
use std::sync::Arc;

pub struct Presenter {
    command_buffer: CommandBuffer,
    semaphores: Semaphores,
    in_flight_fence: Fence,
    swapchain: Swapchain,
}

struct Semaphores {
    image_available: Semaphore,
    render_finished: Semaphore,
}

impl Presenter {
    pub fn new(device: &Arc<Device>, surface: Surface) -> Presenter {
        let swapchain = Swapchain::new(device, surface);

        Presenter {
            semaphores: Semaphores {
                image_available: Semaphore::new(swapchain.device().clone()),
                render_finished: Semaphore::new(swapchain.device().clone()),
            },
            in_flight_fence: Fence::new(swapchain.device().clone(), true),
            command_buffer: CommandPool::new(swapchain.device().graphics_queue()).allocate_buffer(),
            swapchain,
        }
    }

    pub fn format(&self) -> vk::Format {
        self.swapchain.format()
    }

    pub fn draw<F>(&self, func: F)
    where
        F: FnOnce(&CommandBufferRecorder, &SwapchainImageView),
    {
        self.in_flight_fence.wait_and_reset();

        let (image_index, image_view) = self
            .swapchain
            .acquire_next_image(&self.semaphores.image_available);

        self.command_buffer.record(true, |cmd| {
            func(cmd, image_view);
        });

        unsafe {
            let wait_semaphores: Vec<vk::Semaphore> = vec![self.semaphores.image_available.vk()];
            let render_finished: Vec<vk::Semaphore> = vec![self.semaphores.render_finished.vk()];
            let command_buffers: Vec<vk::CommandBuffer> = vec![self.command_buffer.raw()];
            let color_attachment_stage = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let submits = [vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&color_attachment_stage)
                .command_buffers(&command_buffers)
                .signal_semaphores(&render_finished)
                .build()];

            self.swapchain
                .device()
                .raw()
                .queue_submit(
                    self.swapchain.device().graphics_queue().raw(),
                    &submits,
                    self.in_flight_fence.vk(),
                )
                .unwrap();

            let swapchains: Vec<vk::SwapchainKHR> = vec![self.swapchain.raw()];
            let image_indices = vec![image_index];

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(&render_finished)
                .swapchains(&swapchains)
                .image_indices(&image_indices)
                .build();

            self.swapchain
                .device()
                .ext()
                .swapchain
                .queue_present(self.swapchain.device().graphics_queue().raw(), &present)
                .unwrap();
        }
    }
}
