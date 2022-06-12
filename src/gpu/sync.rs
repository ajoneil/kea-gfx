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
