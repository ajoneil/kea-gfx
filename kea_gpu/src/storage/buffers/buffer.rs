use super::{TransferBuffer, UnallocatedBuffer};
use crate::{device::Device, storage::memory::Allocation};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::{mem, slice, sync::Arc};

pub struct Buffer {
    name: String,
    allocation: Allocation,
    buffer: UnallocatedBuffer,
    location: MemoryLocation,
}

impl Buffer {
    pub fn new(
        device: Arc<Device>,
        size: u64,
        usage: vk::BufferUsageFlags,
        name: String,
        location: MemoryLocation,
        alignment: Option<u64>,
    ) -> Buffer {
        UnallocatedBuffer::new(device, size, usage).allocate(name, location, alignment)
    }

    pub fn new_from_data<T: Copy>(
        device: Arc<Device>,
        data: &[T],
        usage: vk::BufferUsageFlags,
        name: String,
        location: MemoryLocation,
        alignment: Option<u64>,
    ) -> Buffer {
        let size = mem::size_of_val(data);
        if location == MemoryLocation::CpuToGpu {
            let mut buffer = Buffer::new(
                device,
                size as _,
                usage,
                name,
                MemoryLocation::CpuToGpu,
                alignment,
            );

            buffer.fill(data);

            buffer
        } else if location == MemoryLocation::GpuOnly {
            let mut buffer = TransferBuffer::new(device, size as _, usage, name, alignment);
            buffer.cpu_buffer().fill(data);

            buffer.transfer_to_gpu()
        } else {
            panic!("Dont't know how to handle memory location");
        }
    }

    pub unsafe fn from_bound_allocation(
        name: String,
        buffer: UnallocatedBuffer,
        allocation: Allocation,
        location: MemoryLocation,
    ) -> Self {
        Self {
            name,
            buffer,
            allocation,
            location,
        }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        let address = unsafe {
            let info = vk::BufferDeviceAddressInfo::builder().buffer(self.buffer.raw());
            self.buffer.device().raw().get_buffer_device_address(&info)
        };
        log::debug!("mem address for buffer  {}: {}", self.name, address);

        address
    }

    pub fn size(&self) -> usize {
        self.buffer.size()
    }

    pub fn allocated_size(&self) -> usize {
        self.allocation.size() as usize
    }

    pub fn count<T>(&self) -> usize {
        self.size() / mem::size_of::<T>()
    }

    pub fn fill<T: Copy>(&mut self, data: &[T]) {
        let data = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<T>())
        };

        assert!(data.len() == self.buffer.size());

        unsafe {
            let slice = &mut self.allocation.mapped_slice_mut()[..data.len()];
            slice.copy_from_slice(data);
        }
    }

    pub fn buffer(&self) -> &UnallocatedBuffer {
        &self.buffer
    }

    pub fn device(&self) -> &Arc<Device> {
        self.buffer.device()
    }

    pub fn location(&self) -> MemoryLocation {
        self.location
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub unsafe fn raw(&self) -> vk::Buffer {
        self.buffer.raw()
    }
}
