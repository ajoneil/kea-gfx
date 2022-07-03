use crate::{device::Device, storage::buffers::AlignedBuffer};
use ash::vk;
use std::sync::Arc;

pub struct ScratchBuffer {
    buffer: AlignedBuffer,
}

impl ScratchBuffer {
    pub fn new(device: Arc<Device>, size: u64) -> ScratchBuffer {
        let vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
            min_acceleration_structure_scratch_offset_alignment: alignment,
            ..
        } = device.physical_device().acceleration_structure_properties();

        let buffer = AlignedBuffer::new(
            device,
            size,
            alignment,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "acceleration structure build scratch".to_string(),
        );

        ScratchBuffer { buffer }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }
}
