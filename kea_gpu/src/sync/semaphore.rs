use crate::device::Device;
use ash::vk;
use std::{slice, sync::Arc};

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

pub struct TimelineSemaphore {
    inner: Semaphore,
}

impl TimelineSemaphore {
    pub fn new(device: Arc<Device>, initial_value: u64) -> TimelineSemaphore {
        let mut type_info = vk::SemaphoreTypeCreateInfo::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(initial_value);
        let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut type_info);

        let raw = unsafe { device.raw().create_semaphore(&create_info, None) }.unwrap();

        log::debug!("created timeline semaphore {:?}", raw);

        TimelineSemaphore {
            inner: Semaphore { raw, device },
        }
    }

    pub fn semaphore(&self) -> &Semaphore {
        &self.inner
    }

    pub fn wait(&self, value: u64) {
        let raw = unsafe { self.inner.raw() };
        let info = vk::SemaphoreWaitInfo::default()
            .semaphores(slice::from_ref(&raw))
            .values(slice::from_ref(&value));
        unsafe {
            self.inner
                .device
                .raw()
                .wait_semaphores(&info, u64::MAX)
                .unwrap();
        }
    }
}
