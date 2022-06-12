use std::sync::Arc;

use ash::vk;

use super::Device;

pub struct Semaphore {
    vk: vk::Semaphore,
    device: Arc<Device>,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Semaphore {
        let semaphore = unsafe {
            device
                .vk()
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();

        Semaphore {
            vk: semaphore,
            device,
        }
    }

    pub unsafe fn vk(&self) -> vk::Semaphore {
        self.vk
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_semaphore(self.vk, None);
        }
    }
}

pub struct Fence {
    vk: vk::Fence,
    device: Arc<Device>,
}

impl Fence {
    pub fn new(device: Arc<Device>, signaled: bool) -> Fence {
        let fence = unsafe {
            device.vk().create_fence(
                &vk::FenceCreateInfo::builder().flags(if signaled {
                    vk::FenceCreateFlags::SIGNALED
                } else {
                    vk::FenceCreateFlags::empty()
                }),
                None,
            )
        }
        .unwrap();

        Fence { vk: fence, device }
    }

    pub unsafe fn vk(&self) -> vk::Fence {
        self.vk
    }

    pub fn wait(&self) {
        unsafe {
            self.device
                .vk()
                .wait_for_fences(&[self.vk], true, u64::MAX)
                .unwrap();
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.vk().reset_fences(&[self.vk]).unwrap();
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_fence(self.vk, None);
        }
    }
}
