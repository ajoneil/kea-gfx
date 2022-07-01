use crate::{device::Device, storage::buffers::AllocatedBuffer};
use ash::vk;
use std::sync::Arc;

pub struct AccelerationStructure {
    device: Arc<Device>,
    buffer: AllocatedBuffer,
    raw: vk::AccelerationStructureKHR,
}

impl AccelerationStructure {
    pub fn new(
        device: &Arc<Device>,
        buffer: AllocatedBuffer,
        ty: vk::AccelerationStructureTypeKHR,
    ) -> AccelerationStructure {
        let raw = unsafe {
            let create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .buffer(buffer.buffer().raw())
                .size(buffer.buffer().size() as u64)
                .ty(ty);

            device
                .ext()
                .acceleration_structure()
                .create_acceleration_structure(&create_info, None)
        }
        .unwrap();

        AccelerationStructure {
            device: device.clone(),
            raw,
            buffer,
        }
    }

    pub fn buffer(&self) -> &AllocatedBuffer {
        &self.buffer
    }

    pub unsafe fn raw(&self) -> vk::AccelerationStructureKHR {
        self.raw
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        let info = vk::AccelerationStructureDeviceAddressInfoKHR::builder()
            .acceleration_structure(self.raw);
        unsafe {
            self.device
                .ext()
                .acceleration_structure()
                .get_acceleration_structure_device_address(&info)
        }
    }
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ext()
                .acceleration_structure()
                .destroy_acceleration_structure(self.raw, None)
        }
    }
}
