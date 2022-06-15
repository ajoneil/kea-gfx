use std::sync::Arc;

use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, MemoryLocation};

use super::Device;

pub struct Buffer {
    device: Arc<Device>,
    vk: vk::Buffer,
}

pub struct AllocatedBuffer {
    buffer: Buffer,
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

    pub fn allocate(self, name: &str, location: MemoryLocation, linear: bool) -> AllocatedBuffer {
        let requirements = unsafe { self.device.vk().get_buffer_memory_requirements(self.vk) };

        let allocation = self
            .device
            .allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear,
            })
            .unwrap();

        unsafe {
            self.device
                .vk()
                .bind_buffer_memory(self.vk, allocation.memory(), allocation.offset())
                .unwrap()
        }

        AllocatedBuffer { buffer: self }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_buffer(self.vk, None);
        }
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.buffer.device.vk().destroy_buffer(self.buffer.vk, None);
        }
    }
}
