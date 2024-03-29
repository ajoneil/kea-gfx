use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct Fence {
    device: Arc<Device>,
    _name: String,
    raw: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<Device>, name: String, signaled: bool) -> Fence {
        let raw = unsafe {
            device.raw().create_fence(
                &vk::FenceCreateInfo::builder().flags(if signaled {
                    vk::FenceCreateFlags::SIGNALED
                } else {
                    vk::FenceCreateFlags::empty()
                }),
                None,
            )
        }
        .unwrap();

        Fence {
            device,
            _name: name,
            raw,
        }
    }

    pub unsafe fn raw(&self) -> vk::Fence {
        self.raw
    }

    pub fn wait(&self) {
        // log::debug!("Waiting on fence {}", self.name);
        unsafe {
            self.device
                .raw()
                .wait_for_fences(&[self.raw], true, u64::MAX)
                .unwrap();
        }
        // log::debug!("Fence {} wait complete", self.name);
    }

    pub fn reset(&self) {
        unsafe {
            self.device.raw().reset_fences(&[self.raw]).unwrap();
        }
    }

    pub fn wait_and_reset(&self) {
        self.wait();
        self.reset();
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_fence(self.raw, None);
        }
    }
}
