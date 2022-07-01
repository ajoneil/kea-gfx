use crate::{
    device::Device,
    storage::{
        buffers::{AllocatedBuffer, Buffer},
        memory,
    },
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub struct ScratchBuffer {
    buffer: AllocatedBuffer,
    size: u64,
    alignment: u32,
}

impl ScratchBuffer {
    pub fn new(device: Arc<Device>, size: u64) -> ScratchBuffer {
        let vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
            min_acceleration_structure_scratch_offset_alignment: alignment,
            ..
        } = device.physical_device().acceleration_structure_properties();

        // The buffer that is created may not meet the alignment requirements
        // of the scratch, so we need the extra space to allow for passing an
        // appropriately aligned address.
        let max_size = size + alignment as u64;

        let buffer = Buffer::new(
            device,
            max_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate(
            "acceleration structure build scratch",
            MemoryLocation::GpuOnly,
        );

        ScratchBuffer {
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
}
