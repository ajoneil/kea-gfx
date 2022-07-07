use crate::{
    commands::RecordedCommandBuffer,
    device::Device,
    storage::images::ImageView,
    sync::{Fence, Semaphore},
};
use ash::vk;
use std::{slice, sync::Arc};

use super::{swapchain::Swapchain, Surface};

pub struct Presenter {
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

    pub fn get_swapchain_image(&self) -> (u32, &ImageView) {
        self.in_flight_fence.wait_and_reset();

        let (index, image) = self
            .swapchain
            .acquire_next_image(&self.semaphores.image_available);

        (index, image)
    }

    pub fn draw(&self, swapchain_index: u32, commands: &[RecordedCommandBuffer]) {
        let raw_commands: Vec<vk::CommandBuffer> =
            commands.iter().map(|c| unsafe { c.raw() }).collect();

        unsafe {
            let image_available = self.semaphores.image_available.vk();
            let render_finished = self.semaphores.render_finished.vk();

            let submit = vk::SubmitInfo::builder()
                .wait_semaphores(slice::from_ref(&image_available))
                .wait_dst_stage_mask(slice::from_ref(
                    &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ))
                .command_buffers(&raw_commands)
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

            let swapchain_raw = self.swapchain.raw();
            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(slice::from_ref(&render_finished))
                .swapchains(slice::from_ref(&swapchain_raw))
                .image_indices(slice::from_ref(&swapchain_index));

            self.swapchain
                .device()
                .ext()
                .swapchain()
                .queue_present(self.swapchain.device().graphics_queue().raw(), &present)
                .unwrap();
        }
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        self.in_flight_fence.wait();
    }
}
