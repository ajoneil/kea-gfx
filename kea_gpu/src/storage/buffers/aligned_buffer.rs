use crate::{
    device::Device,
    storage::{buffers::Buffer, memory},
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::fmt::Debug;
use std::sync::Arc;

pub struct AlignedBuffer {
    buffer: Buffer,
    size: u64,
    alignment: u32,
}

impl AlignedBuffer {
    pub fn new(
        device: Arc<Device>,
        size: u64,
        alignment: u32,
        usage: vk::BufferUsageFlags,
        name: String,
    ) -> AlignedBuffer {
        // The buffer that is created may not meet the alignment requirements
        // of the scratch, so we need the extra space to allow for passing an
        // appropriately aligned address.
        let max_size = size + alignment as u64;

        let buffer = Buffer::new(device, max_size, usage, name, MemoryLocation::GpuOnly);

        AlignedBuffer {
            buffer,
            size,
            alignment,
        }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        let address = self.buffer.device_address();
        let aligned_address = memory::align(address, self.alignment as _);

        assert!(
            (aligned_address + self.size as u64) <= (address + self.buffer.allocated_size() as u64)
        );

        aligned_address
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub unsafe fn raw(&self) -> vk::Buffer {
        self.buffer.raw()
    }
}

impl Debug for AlignedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AlignedBuffer({}|{}=>{})",
            self.buffer.device_address(),
            self.alignment,
            self.device_address()
        )
    }
}
