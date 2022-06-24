use super::device::Device;
use ash::vk;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc},
    MemoryLocation,
};
use std::{
    mem::{self, ManuallyDrop},
    slice,
    sync::Arc,
};

pub struct Buffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    size: u64,
}

pub struct AllocatedBuffer {
    buffer: Buffer,
    allocation: ManuallyDrop<Allocation>,
}

impl Buffer {
    pub fn new(device: Arc<Device>, size: u64, usage: vk::BufferUsageFlags) -> Buffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let raw = unsafe { device.vk().create_buffer(&buffer_info, None) }.unwrap();

        Buffer { device, raw, size }
    }

    pub fn allocate(self, name: &str, location: MemoryLocation, linear: bool) -> AllocatedBuffer {
        let requirements = unsafe { self.device.vk().get_buffer_memory_requirements(self.raw) };

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
                .bind_buffer_memory(self.raw, allocation.memory(), allocation.offset())
                .unwrap()
        }

        AllocatedBuffer {
            buffer: self,
            allocation: ManuallyDrop::new(allocation),
        }
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub unsafe fn raw(&self) -> vk::Buffer {
        self.raw
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_buffer(self.raw, None);
        }
    }
}

impl AllocatedBuffer {
    pub fn device_address(&self) -> vk::DeviceAddress {
        unsafe {
            self.buffer.device.vk().get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder().buffer(self.buffer.raw),
            )
        }
    }

    pub fn allocated_size(&self) -> usize {
        self.allocation.size() as usize
    }

    pub fn fill<T>(&self, data: &[T]) {
        let data = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<T>())
        };
        assert!(data.len() <= self.allocated_size());

        unsafe {
            let pointer = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
            pointer.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.buffer
                .device
                .allocator
                .lock()
                .unwrap()
                .free(ManuallyDrop::take(&mut self.allocation))
                .unwrap();
        }
    }
}
