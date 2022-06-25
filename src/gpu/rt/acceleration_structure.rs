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
                .stride(mem::size_of::<vk::AabbPositionsKHR>() as u64)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .geometry(geometry_data)
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

pub struct AccelerationStructureDescription<'a> {
    ty: vk::AccelerationStructureTypeKHR,
    geometries: Vec<vk::AccelerationStructureGeometryKHR>,
    ranges: Vec<vk::AccelerationStructureBuildRangeInfoKHR>,

    // Used to time the lifetime to the lifetime of the allocated buffer
    marker: PhantomData<&'a ()>,
}

pub struct BuildSizes {
    pub acceleration_structure: vk::DeviceSize,
    pub build_scratch: vk::DeviceSize,
}

impl<'a> AccelerationStructureDescription<'a> {
    pub fn new(
        ty: vk::AccelerationStructureTypeKHR,
        geometries: &[Geometry<'a>],
    ) -> AccelerationStructureDescription<'a> {
        AccelerationStructureDescription {
            ty,
            geometries: geometries.iter().map(|g: &Geometry| g.geometry).collect(),
            ranges: geometries.iter().map(|g: &Geometry| g.range).collect(),
            marker: PhantomData,
        }
    }

    pub fn build_sizes(&self, device: &Device) -> BuildSizes {
        let primitive_counts: Vec<u32> = self.ranges.iter().map(|r| r.primitive_count).collect();

        let vk::AccelerationStructureBuildSizesInfoKHR {
            acceleration_structure_size,
            build_scratch_size,
            ..
        } = unsafe {
            device
                .ext
                .acceleration_structure
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &self.geometry_info(),
                    &primitive_counts,
                )
        };

        BuildSizes {
            acceleration_structure: acceleration_structure_size,
            build_scratch: build_scratch_size,
        }
    }

    pub fn geometry_info(&self) -> vk::AccelerationStructureBuildGeometryInfoKHRBuilder {
        vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(self.ty)
            .geometries(&self.geometries)
    }

    pub fn bind_for_build(
        &'a self,
        destination: &'a AccelerationStructure,
        scratch: &'a AllocatedBuffer,
    ) -> BoundAccelerationStructureDescription<'a> {
        BoundAccelerationStructureDescription {
            acceleration_structure_description: self,
            destination,
            scratch,
        }
    }
}

pub struct BoundAccelerationStructureDescription<'a> {
    acceleration_structure_description: &'a AccelerationStructureDescription<'a>,
    destination: &'a AccelerationStructure,
    scratch: &'a AllocatedBuffer,
}

impl<'a> BoundAccelerationStructureDescription<'a> {
    pub fn geometry_info(&self) -> vk::AccelerationStructureBuildGeometryInfoKHR {
        self.acceleration_structure_description
            .geometry_info()
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .dst_acceleration_structure(self.destination.raw)
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: self.scratch.device_address(),
            })
            .build()
    }

    pub fn ranges(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        &self.acceleration_structure_description.ranges
    }
}

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
                .ext
                .acceleration_structure
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
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ext
                .acceleration_structure
                .destroy_acceleration_structure(self.raw, None)
        }
    }
}
