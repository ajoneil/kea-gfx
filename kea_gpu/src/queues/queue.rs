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
        let wait_infos: Vec<vk::SemaphoreSubmitInfo> = submission
            .wait
            .iter()
            .map(|w| {
                vk::SemaphoreSubmitInfo::default()
                    .semaphore(unsafe { w.semaphore.raw() })
                    .stage_mask(w.stage)
                    .value(w.value)
            })
            .collect();

        let command_infos: Vec<vk::CommandBufferSubmitInfo> = submission
            .commands
            .iter()
            .map(|c| vk::CommandBufferSubmitInfo::default().command_buffer(unsafe { c.raw() }))
            .collect();

        let signal_infos: Vec<vk::SemaphoreSubmitInfo> = submission
            .signal
            .iter()
            .map(|s| {
                vk::SemaphoreSubmitInfo::default()
                    .semaphore(unsafe { s.semaphore.raw() })
                    .stage_mask(s.stage)
                    .value(s.value)
            })
            .collect();

        let submit_info = vk::SubmitInfo2::default()
            .wait_semaphore_infos(&wait_infos)
            .command_buffer_infos(&command_infos)
            .signal_semaphore_infos(&signal_infos);

        let raw_fence = fence.map_or(vk::Fence::null(), |f| unsafe { f.raw() });

        unsafe {
            self.device
                .raw()
                .queue_submit2(self.raw(), slice::from_ref(&submit_info), raw_fence)
                .unwrap();
        }
    }
}
