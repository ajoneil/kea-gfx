use crate::{device::Device, storage::buffers::Buffer};
use ash::vk;
use std::{slice, sync::Arc};

pub struct AccelerationStructure {
    device: Arc<Device>,
    buffer: Buffer,
    raw: vk::AccelerationStructureKHR,
}

impl AccelerationStructure {
    pub fn new(
        device: &Arc<Device>,
        buffer: Buffer,
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

    pub fn buffer(&self) -> &Buffer {
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

    pub fn build_sizes(
        device: &Device,
        geometry_info: vk::AccelerationStructureBuildGeometryInfoKHR,
        range: vk::AccelerationStructureBuildRangeInfoKHR,
    ) -> BuildSizes {
        let primitive_count = range.primitive_count;

        let vk::AccelerationStructureBuildSizesInfoKHR {
            acceleration_structure_size,
            build_scratch_size,
            ..
        } = unsafe {
            device
                .ext()
                .acceleration_structure()
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &geometry_info,
                    slice::from_ref(&primitive_count),
                )
        };

        BuildSizes {
            acceleration_structure: acceleration_structure_size,
            build_scratch: build_scratch_size,
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

pub struct BuildSizes {
    pub acceleration_structure: vk::DeviceSize,
    pub build_scratch: vk::DeviceSize,
}
