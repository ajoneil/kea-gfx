use crate::{
    commands::RecordedCommandBuffer,
    device::Device,
    queues::{Signal, Submission, Wait},
    storage::images::ImageView,
    sync::{Semaphore, TimelineSemaphore},
};
use ash::vk;
use std::{cell::Cell, slice, sync::Arc};

use super::{swapchain::Swapchain, Surface};

/// Maximum number of frames the CPU may queue ahead of the GPU.
pub const FRAMES_IN_FLIGHT: u64 = 2;

pub struct Presenter {
    swapchain: Swapchain,
    /// One acquire semaphore per in-flight frame slot.
    acquire_semaphores: Vec<Semaphore>,
    /// One present semaphore per swapchain image.
    present_semaphores: Vec<Semaphore>,
    /// Tracks GPU completion of each submitted frame. Frame N signals value N+1.
    timeline: TimelineSemaphore,
    /// Index of the next frame to record. Incremented after each submit.
    frame_index: Cell<u64>,
}

impl Presenter {
    pub fn new(device: &Arc<Device>, surface: Surface, size: (u32, u32)) -> Presenter {
        let extent = vk::Extent2D {
            width: size.0,
            height: size.1,
        };
        let swapchain = Swapchain::new(device, surface, extent);

        let acquire_semaphores = (0..FRAMES_IN_FLIGHT)
            .map(|i| Semaphore::new_named(device.clone(), &format!("acquire {}", i)))
            .collect();
        let present_semaphores = (0..swapchain.image_count())
            .map(|i| Semaphore::new_named(device.clone(), &format!("present {}", i)))
            .collect();
        let timeline = TimelineSemaphore::new_named(device.clone(), 0, "frame timeline");

        Presenter {
            swapchain,
            acquire_semaphores,
            present_semaphores,
            timeline,
            frame_index: Cell::new(0),
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

    /// Index of the frame currently being recorded. Stable across
    /// get_swapchain_image and draw calls within a frame; increments after draw.
    pub fn frame_index(&self) -> u64 {
        self.frame_index.get()
    }

    pub fn get_swapchain_image(&self) -> (u32, &ImageView) {
        let frame = self.frame_index.get();
        // Wait for frame (frame - FRAMES_IN_FLIGHT) to finish on the GPU before
        // reusing its acquire semaphore slot. Frame N signals timeline = N + 1,
        // so the wait value is (frame - FRAMES_IN_FLIGHT) + 1.
        if frame >= FRAMES_IN_FLIGHT {
            self.timeline.wait(frame - FRAMES_IN_FLIGHT + 1);
        }

        let acquire = &self.acquire_semaphores[(frame % FRAMES_IN_FLIGHT) as usize];
        self.swapchain.acquire_next_image(acquire)
    }

    pub fn draw(&self, swapchain_index: u32, commands: &[RecordedCommandBuffer]) {
        let frame = self.frame_index.get();
        let acquire = &self.acquire_semaphores[(frame % FRAMES_IN_FLIGHT) as usize];
        let present = &self.present_semaphores[swapchain_index as usize];

        let wait = Wait {
            semaphore: acquire,
            stage: vk::PipelineStageFlags2::TRANSFER,
            value: 0,
        };

        let signals = [
            Signal {
                semaphore: present,
                stage: vk::PipelineStageFlags2::ALL_COMMANDS,
                value: 0,
            },
            Signal {
                semaphore: self.timeline.semaphore(),
                stage: vk::PipelineStageFlags2::ALL_COMMANDS,
                value: frame + 1,
            },
        ];

        let submission = Submission {
            wait: slice::from_ref(&wait),
            commands: &commands,
            signal: &signals,
        };

        self.swapchain
            .device()
            .graphics_queue()
            .submit(&submission, None);

        self.swapchain.present(
            &self.swapchain.device().graphics_queue(),
            slice::from_ref(present),
            swapchain_index,
        );

        self.frame_index.set(frame + 1);
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        let frame = self.frame_index.get();
        if frame > 0 {
            self.timeline.wait(frame);
        }
    }
}
