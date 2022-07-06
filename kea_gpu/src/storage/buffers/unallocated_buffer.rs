use super::Buffer;
use crate::{device::Device, storage::memory::Allocation};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub struct UnallocatedBuffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    size: u64,
}

impl UnallocatedBuffer {
    pub fn new(device: Arc<Device>, size: u64, usage: vk::BufferUsageFlags) -> UnallocatedBuffer {
        let usage = usage | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS;
        let buffer_info = vk::BufferCreateInfo::builder().size(size).usage(usage);
        let raw = unsafe { device.raw().create_buffer(&buffer_info, None) }.unwrap();

        UnallocatedBuffer { device, raw, size }
    }

    pub fn allocate(
        self,
        name: String,
        location: MemoryLocation,
        alignment: Option<u64>,
    ) -> Buffer {
        let mut requirements =
            unsafe { self.device().raw().get_buffer_memory_requirements(self.raw) };
        if let Some(alignment) = alignment {
            requirements.alignment = requirements.alignment.max(alignment);
        }

        let allocation = Allocation::new(self.device.clone(), name.clone(), location, requirements);

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
