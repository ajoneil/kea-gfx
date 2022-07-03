use crate::{commands::CommandBuffer, device::Device, storage::buffers::Buffer};
use ash::vk;
use gpu_allocator::MemoryLocation;
use kea_gpu_shaderlib::Aabb;
use std::{mem, slice, sync::Arc};

use super::{acceleration_structure::AccelerationStructure, scratch_buffer::ScratchBuffer};

pub struct Geometry {
    // type: AABB(intersection_shader: EntryPoint)
    // hit_shader: EntryPoint
    name: String,
    geometry_data: Buffer,
    additional_data: Arc<Buffer>,
    acceleration_structure: Option<Arc<AccelerationStructure>>,
}

impl Geometry {
    pub fn new(name: String, geometry_data: Buffer, additional_data: Arc<Buffer>) -> Self {
        Self {
            name,
            geometry_data,
            additional_data,
            acceleration_structure: None,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        self.geometry_data.device()
    }

    pub fn acceleration_structure(&self) -> &Arc<AccelerationStructure> {
        if self.acceleration_structure.is_none() {
            panic!("Geometry {} isn't built", self.name);
        }

        self.acceleration_structure.as_ref().unwrap()
    }

    pub fn additional_data(&self) -> &Arc<Buffer> {
        &self.additional_data
    }

    pub fn build(&mut self) {
        if self.acceleration_structure.is_some() {
            log::warn!("Geometry {} has multiple build calls.", self.name);
        }

        let geometry_data = vk::AccelerationStructureGeometryDataKHR {
            aabbs: vk::AccelerationStructureGeometryAabbsDataKHR::builder()
                .data(vk::DeviceOrHostAddressConstKHR {
                    device_address: self.geometry_data.device_address(),
                })
                .stride(mem::size_of::<Aabb>() as u64)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .geometry(geometry_data)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .build();

        let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count((self.geometry_data.buffer().size() / mem::size_of::<Aabb>()) as u32)
            .build();

        let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .geometries(slice::from_ref(&geometry))
            .build();

        let build_sizes = AccelerationStructure::build_sizes(self.device(), geometry_info, range);
        let scratch_buffer = ScratchBuffer::new(self.device().clone(), build_sizes.build_scratch);

        let acceleration_structure_buffer = Buffer::new(
            self.device().clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            format!("{} acceleration structure", self.name),
            MemoryLocation::GpuOnly,
        );

        let acceleration_structure = AccelerationStructure::new(
            self.device(),
            acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
        );

        let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .geometries(slice::from_ref(&geometry))
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .dst_acceleration_structure(unsafe { acceleration_structure.raw() })
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: scratch_buffer.device_address(),
            })
            .build();

        CommandBuffer::now(self.device(), "build BLAS".to_string(), |cmd| {
            cmd.build_acceleration_structure(geometry_info, range);
        });

        self.acceleration_structure = Some(Arc::new(acceleration_structure));
    }
}
