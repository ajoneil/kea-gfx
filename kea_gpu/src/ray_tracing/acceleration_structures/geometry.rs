use crate::storage::buffers::Buffer;
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::{marker::PhantomData, mem};

pub struct Geometry<'a> {
    pub geometry: vk::AccelerationStructureGeometryKHR,
    pub range: vk::AccelerationStructureBuildRangeInfoKHR,

    // Used to time the lifetime to the lifetime of the allocated buffer
    marker: PhantomData<&'a ()>,
}

impl<'a> Geometry<'a> {
    pub fn aabbs(buffer: &'a Buffer) -> Geometry<'a> {
        if buffer.location() != MemoryLocation::GpuOnly {
            log::warn!(
                "Buffer {} is used for AABBs but it's not in exclusive GPU memory - \
                 unless you're debugging use a TransferBuffer to move the data to \
                 the GPU first",
                buffer.name()
            )
        }

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

    pub fn instances(buffer: &'a Buffer) -> Geometry<'a> {
        if buffer.location() != MemoryLocation::GpuOnly {
            log::warn!(
                "Buffer {} is used for accel struct instances but it's not in exclusive \
                 GPU memory - unless you're debugging use a TransferBuffer to move the \
                 data to the GPU first",
                buffer.name()
            )
        }

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
