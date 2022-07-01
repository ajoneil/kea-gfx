use super::AllocatedBuffer;
use crate::device::Device;
use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, MemoryLocation};
use std::sync::Arc;

pub struct Buffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    size: u64,
}

impl Buffer {
    pub fn new(device: Arc<Device>, size: u64, usage: vk::BufferUsageFlags) -> Buffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let raw = unsafe { device.raw().create_buffer(&buffer_info, None) }.unwrap();

        Buffer { device, raw, size }
    }

    pub fn allocate(self, name: &str, location: MemoryLocation) -> AllocatedBuffer {
        let requirements = unsafe { self.device().raw().get_buffer_memory_requirements(self.raw) };

        self.allocate_with_mem_requirements(name, location, requirements)
    }

    fn allocate_with_mem_requirements(
        self,
        name: &str,
        location: MemoryLocation,
        requirements: vk::MemoryRequirements,
    ) -> AllocatedBuffer {
        let allocation = self
            .device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        unsafe {
            self.device
                .raw()
                .bind_buffer_memory(self.raw, allocation.memory(), allocation.offset())
                .unwrap()
        }

        AllocatedBuffer::new(name.to_string(), self, allocation)
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub unsafe fn raw(&self) -> vk::Buffer {
        self.raw
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_buffer(self.raw, None);
        }
    }
}
