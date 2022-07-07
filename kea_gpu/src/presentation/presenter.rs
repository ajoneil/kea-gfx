use crate::{
    commands::RecordedCommandBuffer,
    device::Device,
    queues::{Submission, Wait},
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
        let wait = Wait {
            semaphore: &self.semaphores.image_available,
            stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        };

        let submission = Submission {
            wait: slice::from_ref(&wait),
            commands: &commands,
            signal_semaphores: slice::from_ref(&self.semaphores.render_finished),
        };

        self.swapchain
            .device()
            .graphics_queue()
            .submit(&submission, Some(&self.in_flight_fence));

        self.swapchain.present(
            &self.swapchain.device().graphics_queue(),
            slice::from_ref(&self.semaphores.render_finished),
            swapchain_index,
        );
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        self.in_flight_fence.wait();
    }
}
