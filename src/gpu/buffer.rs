use std::sync::Arc;

use ash::vk;

use super::Device;

pub struct Buffer {
    device: Arc<Device>,
    vk: vk::Buffer,
}

impl Buffer {
    pub fn new(device: &Arc<Device>, size: u64, usage: vk::BufferUsageFlags) -> Buffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.vk().create_buffer(&buffer_info, None) }.unwrap();

        Buffer {
            device: device.clone(),
            vk: buffer,
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_buffer(self.vk, None);
        }
    }
}
