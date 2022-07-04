use crate::{
    commands::{CommandBufferRecorder, CommandPool},
    device::Device,
    storage::images::ImageView,
    sync::{Fence, Semaphore},
};
use ash::vk;
use std::{slice, sync::Arc};

use super::{swapchain::Swapchain, Surface};

pub struct Presenter {
    command_pool: Arc<CommandPool>,
    semaphores: Semaphores,
    in_flight_fence: Fence,
    swapchain: Swapchain,
}

struct Semaphores {
    image_available: Semaphore,
    render_finished: Semaphore,
}

impl Presenter {
    pub fn new(device: &Arc<Device>, surface: Surface, size: (u32, u32)) -> Presenter {
        let extent = vk::Extent2D {
            width: size.0,
            height: size.1,
        };
        let swapchain = Swapchain::new(device, surface, extent);

        Presenter {
            semaphores: Semaphores {
                image_available: Semaphore::new(swapchain.device().clone()),
                render_finished: Semaphore::new(swapchain.device().clone()),
            },
            in_flight_fence: Fence::new(swapchain.device().clone(), "in flight".to_string(), true),
            command_pool: CommandPool::new(swapchain.device().graphics_queue()),
            swapchain,
        }
    }

    pub fn format(&self) -> vk::Format {
        self.swapchain.format()
    }

    pub fn size(&self) -> (u32, u32) {
        (
            self.swapchain.extent().width,
            self.swapchain.extent().height,
        )
    }

    pub fn draw<F>(&self, func: F)
    where
        F: FnOnce(&CommandBufferRecorder, &ImageView),
    {
        self.in_flight_fence.wait_and_reset();

        let (image_index, image_view) = self
            .swapchain
            .acquire_next_image(&self.semaphores.image_available);

        let cmd = self
            .command_pool
            .allocate_buffer("draw".to_string())
            .record(|cmd| {
                func(cmd, image_view);
            });

        unsafe {
            let image_available = self.semaphores.image_available.vk();
            let render_finished = self.semaphores.render_finished.vk();
            // This would be dangerous if we were destroying commands, but they
            // live as long as their pool.
            let cmd = cmd.consume().raw();

            let submit = vk::SubmitInfo::builder()
                .wait_semaphores(slice::from_ref(&image_available))
                .wait_dst_stage_mask(slice::from_ref(
                    &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ))
                .command_buffers(slice::from_ref(&cmd))
                .signal_semaphores(slice::from_ref(&render_finished));
            // self.in_flight_fence = self.swapchain.device().graphics_queue().submit();

            // log::debug!("Submitting draw command..");

            self.swapchain
                .device()
                .raw()
                .queue_submit(
                    self.swapchain.device().graphics_queue().raw(),
                    slice::from_ref(&submit),
                    self.in_flight_fence.raw(),
                )
                .unwrap();

            // log::debug!("Submission complete");

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(slice::from_ref(&render_finished))
                .swapchains(slice::from_ref(&self.swapchain.raw()))
                .image_indices(slice::from_ref(&image_index))
                .build();

            self.swapchain
                .device()
                .ext()
                .swapchain()
                .queue_present(self.swapchain.device().graphics_queue().raw(), &present)
                .unwrap();
        }
    }
}
