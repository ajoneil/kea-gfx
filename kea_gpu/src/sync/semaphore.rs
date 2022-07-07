use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct Semaphore {
    device: Arc<Device>,
    raw: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Semaphore {
        let raw = unsafe {
            device
                .raw()
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();

        log::debug!("created semaphore {:?}", raw);

        Semaphore { raw, device }
    }

    pub unsafe fn raw(&self) -> vk::Semaphore {
        self.raw
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        log::debug!("destroying semaphore {:?}", self.raw);

        unsafe {
            self.device.raw().destroy_semaphore(self.raw, None);
        }
    }
}
