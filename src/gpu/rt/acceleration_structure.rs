use crate::gpu::{buffer::AllocatedBuffer, device::Device};
use ash::vk::{self};
use glam::Vec3;
use std::{marker::PhantomData, mem, sync::Arc};

#[repr(C)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
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
            .primitive_count((buffer.buffer().size() / mem::size_of::<Aabb>()) as u32)
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

    pub fn build_scratch_size(&self, device: &Device) -> u64 {
        let primitive_counts: Vec<u32> = self.ranges.iter().map(|r| r.primitive_count).collect();

        unsafe {
            device
                .ext
                .acceleration_structure
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &self.geometry_info(),
                    &primitive_counts,
                )
        }
        .build_scratch_size
    }

    pub fn geometry_info(&self) -> vk::AccelerationStructureBuildGeometryInfoKHRBuilder {
        vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .geometries(&self.geometries)
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
        self.blas
            .geometry_info()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .src_acceleration_structure(self.src)
            .dst_acceleration_structure(self.dst)
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: self.scratch.device_address(),
            })
            .build()
    }

    pub fn ranges(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        &self.blas.ranges
    }
}

pub struct AccelerationStructure {
    device: Arc<Device>,
    buffer: AllocatedBuffer,
    vk: vk::AccelerationStructureKHR,
}

impl AccelerationStructure {
    pub fn new(
        device: &Arc<Device>,
        buffer: AllocatedBuffer,
        ty: vk::AccelerationStructureTypeKHR,
    ) -> AccelerationStructure {
        let vk = unsafe {
            let create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .buffer(buffer.buffer().vk())
                .size(buffer.buffer().size() as u64)
                .ty(ty);

            device
                .ext
                .acceleration_structure
                .create_acceleration_structure(&create_info, None)
        }
        .unwrap();

        AccelerationStructure {
            device: device.clone(),
            vk,
            buffer,
        }
    }
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ext
                .acceleration_structure
                .destroy_acceleration_structure(self.vk, None)
        }
    }
}
