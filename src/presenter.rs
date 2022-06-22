use crate::gpu::{
    command::{CommandBuffer, CommandBufferRecorder, CommandPool},
    swapchain::{Swapchain, SwapchainImageView},
    sync::{Fence, Semaphore},
};
use ash::vk;
use std::sync::Arc;

pub struct Presenter {
    swapchain: Swapchain,
    command_buffer: CommandBuffer,
    semaphores: Semaphores,
    in_flight_fence: Fence,
}

struct Semaphores {
    image_available: Semaphore,
    render_finished: Semaphore,
}

impl Presenter {
    pub fn new(swapchain: Swapchain) -> Presenter {
        Presenter {
            semaphores: Semaphores {
                image_available: Semaphore::new(swapchain.device.clone()),
                render_finished: Semaphore::new(swapchain.device.clone()),
            },
            in_flight_fence: Fence::new(swapchain.device.clone(), true),
            command_buffer: Arc::new(CommandPool::new(swapchain.device.queues().graphics()))
                .allocate_buffer(),
            swapchain,
        }
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
                    self.swapchain.device.queues().graphics().vk(),
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
                .queue_present(self.swapchain.device.queues().graphics().vk(), &present)
                .unwrap();
        }
    }
}
