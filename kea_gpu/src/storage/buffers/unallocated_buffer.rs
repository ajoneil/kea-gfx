use super::Buffer;
use crate::device::Device;
use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, MemoryLocation};
use std::sync::Arc;

pub struct UnallocatedBuffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    size: u64,
}

impl UnallocatedBuffer {
    pub fn new(device: Arc<Device>, size: u64, usage: vk::BufferUsageFlags) -> UnallocatedBuffer {
        let usage = usage | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS;
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let raw = unsafe { device.raw().create_buffer(&buffer_info, None) }.unwrap();

        UnallocatedBuffer { device, raw, size }
    }

    pub fn allocate(self, name: String, location: MemoryLocation) -> Buffer {
        let requirements = unsafe { self.device().raw().get_buffer_memory_requirements(self.raw) };

        let allocation = self
            .device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: &name,
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        unsafe {
            self.device
                .raw()
                .bind_buffer_memory(self.raw, allocation.memory(), allocation.offset())
                .unwrap();

            Buffer::from_bound_allocation(name, self, allocation, location)
        }
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub unsafe fn raw(&self) -> vk::Buffer {
        self.raw
    }
}

impl Drop for UnallocatedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_buffer(self.raw, None);
        }
    }
}
