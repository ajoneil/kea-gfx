use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct Pipeline {
    device: Arc<Device>,
    raw: vk::Pipeline,
}

impl Pipeline {
    pub unsafe fn new(device: Arc<Device>, raw: vk::Pipeline) -> Pipeline {
        Pipeline { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::Pipeline {
        self.raw
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline(self.raw, None);
        }
    }
}
