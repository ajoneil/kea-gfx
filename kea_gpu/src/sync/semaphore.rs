use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct Semaphore {
    vk: vk::Semaphore,
    device: Arc<Device>,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Semaphore {
        let vk = unsafe {
            device
                .raw()
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();

        log::debug!("created semaphore {:?}", vk);

        Semaphore { vk, device }
    }

    pub unsafe fn vk(&self) -> vk::Semaphore {
        self.vk
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        log::debug!("destroying semaphore {:?}", self.vk);

        unsafe {
            self.device.raw().destroy_semaphore(self.vk, None);
        }
    }
}
