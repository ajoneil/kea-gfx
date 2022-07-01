use std::sync::Arc;

use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::{
    core::buffer::{AllocatedBuffer, Buffer},
    device::Device,
    storage::memory,
};

pub struct ScratchBuffer {
    buffer: AllocatedBuffer,
    alignment: u32,
}

impl ScratchBuffer {
    pub fn new(device: Arc<Device>, size: u64) -> ScratchBuffer {
        let vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
            min_acceleration_structure_scratch_offset_alignment: alignment,
            ..
        } = device.physical_device().acceleration_structure_properties();

        let aligned_size = memory::align(size, alignment as u64);
        let buffer = Buffer::new(
            device,
            aligned_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        )
        .allocate(
            "acceleration structure build scratch",
            MemoryLocation::GpuOnly,
        );

        ScratchBuffer { buffer, alignment }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        memory::align(self.buffer.device_address(), self.alignment as _)
    }
}
