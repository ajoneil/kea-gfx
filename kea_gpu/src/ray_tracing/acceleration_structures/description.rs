use std::marker::PhantomData;
use ash::vk;
use crate::device::Device;
use super::{Geometry, AccelerationStructure, ScratchBuffer};

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
                .ext()
                .acceleration_structure()
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
        scratch: &'a ScratchBuffer,
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
    scratch: &'a ScratchBuffer,
}

impl<'a> BoundAccelerationStructureDescription<'a> {
    pub fn geometry_info(&self) -> vk::AccelerationStructureBuildGeometryInfoKHR {
        self.acceleration_structure_description
            .geometry_info()
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .dst_acceleration_structure(unsafe { self.destination.raw() })
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: self.scratch.device_address(),
            })
            .build()
    }

    pub fn ranges(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        &self.acceleration_structure_description.ranges
    }
}
