use crate::{
    device::{Device, QueueFamily},
    sync::Fence,
};
use ash::vk;
use std::{slice, sync::Arc};

use super::Submission;

pub struct Queue {
    device: Arc<Device>,
    raw: vk::Queue,
    family: QueueFamily,
}

impl Queue {
    pub unsafe fn new_from_raw(device: Arc<Device>, raw: vk::Queue, family: QueueFamily) -> Self {
        Self {
            device,
            raw,
            family,
        }
    }
    pub unsafe fn raw(&self) -> vk::Queue {
        self.raw
    }

    pub fn family(&self) -> &QueueFamily {
        &self.family
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn submit(&self, submission: &Submission, fence: Option<&Fence>) {
        let raw_wait_semaphores: Vec<vk::Semaphore> = submission
            .wait
            .iter()
            .map(|w| unsafe { w.semaphore.raw() })
            .collect();

        let raw_wait_stages: Vec<vk::PipelineStageFlags> =
            submission.wait.iter().map(|w| w.stage).collect();

        let raw_buffers: Vec<vk::CommandBuffer> = submission
            .commands
            .iter()
            .map(|c| unsafe { c.raw() })
            .collect();

        let raw_signal_semaphores: Vec<vk::Semaphore> = submission
            .signal_semaphores
            .iter()
            .map(|s| unsafe { s.raw() })
            .collect();

        let mut submit_info = vk::SubmitInfo::builder().command_buffers(&raw_buffers);

        if raw_wait_semaphores.len() > 0 {
            submit_info = submit_info
                .wait_semaphores(&raw_wait_semaphores)
                .wait_dst_stage_mask(&raw_wait_stages);
        };

        if raw_signal_semaphores.len() > 0 {
            submit_info = submit_info.signal_semaphores(&raw_signal_semaphores);
        }

        let raw_fence = if let Some(fence) = fence {
            unsafe { fence.raw() }
        } else {
            vk::Fence::null()
        };

        unsafe {
            self.device
                .raw()
                .queue_submit(self.raw(), slice::from_ref(&submit_info), raw_fence)
                .unwrap();
        }
    }
}
