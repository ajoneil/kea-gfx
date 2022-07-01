use crate::storage::buffers::AllocatedBuffer;
use ash::vk;
use std::{marker::PhantomData, mem};

pub struct Geometry<'a> {
    pub geometry: vk::AccelerationStructureGeometryKHR,
    pub range: vk::AccelerationStructureBuildRangeInfoKHR,

    // Used to time the lifetime to the lifetime of the allocated buffer
    marker: PhantomData<&'a ()>,
}

impl<'a> Geometry<'a> {
    pub fn aabbs(buffer: &'a AllocatedBuffer) -> Geometry<'a> {
        let geometry_data = vk::AccelerationStructureGeometryDataKHR {
            aabbs: vk::AccelerationStructureGeometryAabbsDataKHR::builder()
                .data(vk::DeviceOrHostAddressConstKHR {
                    device_address: buffer.device_address(),
                })
                .stride(mem::size_of::<vk::AabbPositionsKHR>() as u64)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .geometry(geometry_data)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .build();

        let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(
                (buffer.buffer().size() / mem::size_of::<vk::AabbPositionsKHR>()) as u32,
            )
            .build();

        Geometry {
            geometry,
            range,
            marker: PhantomData,
        }
    }

    pub fn instances(buffer: &'a AllocatedBuffer) -> Geometry<'a> {
        let geometry_data = vk::AccelerationStructureGeometryDataKHR {
            instances: vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                .data(vk::DeviceOrHostAddressConstKHR {
                    device_address: buffer.device_address(),
                })
                .array_of_pointers(false)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .geometry(geometry_data)
            .build();

        let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(
                (buffer.buffer().size() / mem::size_of::<vk::AccelerationStructureInstanceKHR>())
                    as u32,
            )
            .build();

        Geometry {
            geometry,
            range,
            marker: PhantomData,
        }
    }
}
