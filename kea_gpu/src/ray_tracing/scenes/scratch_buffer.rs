use crate::{device::Device, storage::buffers::Buffer};
use ash::vk;
use std::sync::Arc;

pub struct ScratchBuffer {
    buffer: Buffer,
}

impl ScratchBuffer {
    pub fn new(device: Arc<Device>, size: u64) -> ScratchBuffer {
        let vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
            min_acceleration_structure_scratch_offset_alignment: alignment,
            ..
        } = device.physical_device().acceleration_structure_properties();

        let buffer = Buffer::new(
            device,
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "acceleration structure build scratch".to_string(),
            gpu_allocator::MemoryLocation::GpuOnly,
            Some(alignment as _),
        );

        ScratchBuffer { buffer }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }
}
