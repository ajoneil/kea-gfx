use crate::gpu::buffer::AllocatedBuffer;
use ash::vk::{self};
use glam::Vec3;
use std::{marker::PhantomData, mem};

#[repr(C)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

pub struct Geometry<'a> {
    geometry: vk::AccelerationStructureGeometryKHR,
    range: vk::AccelerationStructureBuildRangeInfoKHR,

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
                .stride(mem::size_of::<Aabb>() as u64)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .geometry(geometry_data)
            .build();

        let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count((buffer.size() / mem::size_of::<Aabb>()) as u32)
            .build();

        Geometry {
            geometry,
            range,
            marker: PhantomData,
        }
    }
}

pub struct Blas<'a> {
    geometries: Vec<vk::AccelerationStructureGeometryKHR>,
    ranges: Vec<vk::AccelerationStructureBuildRangeInfoKHR>,

    // Used to time the lifetime to the lifetime of the allocated buffer
    marker: PhantomData<&'a ()>,
}

impl<'a> Blas<'a> {
    pub fn new(geometries: &[Geometry<'a>]) -> Blas<'a> {
        Blas {
            geometries: geometries.iter().map(|g: &Geometry| g.geometry).collect(),
            ranges: geometries.iter().map(|g: &Geometry| g.range).collect(),
            marker: PhantomData,
        }
    }

    pub fn bind_for_build(
        &'a self,
        src: vk::AccelerationStructureKHR,
        dst: vk::AccelerationStructureKHR,
        scratch: &'a AllocatedBuffer,
    ) -> BoundBlas<'a> {
        BoundBlas {
            blas: self,
            src,
            dst,
            scratch,
        }
    }
}

pub struct BoundBlas<'a> {
    blas: &'a Blas<'a>,
    src: vk::AccelerationStructureKHR,
    dst: vk::AccelerationStructureKHR,
    scratch: &'a AllocatedBuffer,
}

impl<'a> BoundBlas<'a> {
    pub fn geometry_info(&self) -> vk::AccelerationStructureBuildGeometryInfoKHR {
        vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .src_acceleration_structure(self.src)
            .dst_acceleration_structure(self.dst)
            .geometries(&self.blas.geometries)
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: self.scratch.device_address(),
            })
            .build()
    }

    pub fn ranges(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        &self.blas.ranges
    }
}
