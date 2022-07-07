use crate::{commands::CommandBuffer, device::Device, storage::buffers::Buffer};
use ash::vk;
use glam::Vec3;
use gpu_allocator::MemoryLocation;
use kea_gpu_shaderlib::Aabb;
use std::{mem, slice, sync::Arc};

use super::{acceleration_structure::AccelerationStructure, scratch_buffer::ScratchBuffer};

pub enum GeometryType {
    Triangles { vertices: Buffer, indices: Buffer },
    Aabbs(Buffer),
}

pub struct Geometry {
    device: Arc<Device>,
    name: String,
    geometry_type: GeometryType,
    additional_data: Option<Arc<Buffer>>,
    acceleration_structure: Option<Arc<AccelerationStructure>>,
}

impl Geometry {
    pub fn new(
        device: Arc<Device>,
        name: String,
        geometry_type: GeometryType,
        additional_data: Option<Arc<Buffer>>,
    ) -> Self {
        Self {
            device,
            name,
            geometry_type,
            additional_data,
            acceleration_structure: None,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn acceleration_structure(&self) -> &Arc<AccelerationStructure> {
        if self.acceleration_structure.is_none() {
            panic!("Geometry {} isn't built", self.name);
        }

        self.acceleration_structure.as_ref().unwrap()
    }

    pub fn additional_data(&self) -> &Option<Arc<Buffer>> {
        &self.additional_data
    }

    pub fn build(&mut self) {
        if self.acceleration_structure.is_some() {
            log::warn!("Geometry {} has multiple build calls.", self.name);
        }

        self.acceleration_structure = match &self.geometry_type {
            GeometryType::Aabbs(aabbs_buffer) => {
                let aabbs = vk::AccelerationStructureGeometryAabbsDataKHR::builder()
                    .data(vk::DeviceOrHostAddressConstKHR {
                        device_address: aabbs_buffer.device_address(),
                    })
                    .stride(mem::size_of::<Aabb>() as u64);

                let geometry = vk::AccelerationStructureGeometryKHR::builder()
                    .geometry_type(vk::GeometryTypeKHR::AABBS)
                    .geometry(vk::AccelerationStructureGeometryDataKHR { aabbs: *aabbs })
                    .flags(vk::GeometryFlagsKHR::OPAQUE);

                let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
                    .primitive_count((aabbs_buffer.size() / mem::size_of::<Aabb>()) as u32);

                let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                    .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                    .geometries(slice::from_ref(&geometry));

                let build_sizes =
                    AccelerationStructure::build_sizes(self.device(), &geometry_info, &range);
                let scratch_buffer =
                    ScratchBuffer::new(self.device().clone(), build_sizes.build_scratch);

                let acceleration_structure_buffer = Buffer::new(
                    self.device().clone(),
                    build_sizes.acceleration_structure,
                    vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
                    format!("{} acceleration structure", self.name),
                    MemoryLocation::GpuOnly,
                    None,
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
                    });

                CommandBuffer::now(self.device(), "build BLAS".to_string(), |cmd| {
                    cmd.build_acceleration_structure(&geometry_info, &range);
                });

                Some(Arc::new(acceleration_structure))
            }
            GeometryType::Triangles { vertices, indices } => {
                let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_format(vk::Format::R32G32B32_SFLOAT)
                    .vertex_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: vertices.device_address(),
                    })
                    .vertex_stride(mem::size_of::<Vec3>() as _)
                    .index_type(vk::IndexType::UINT16)
                    .index_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: indices.device_address(),
                    })
                    .max_vertex(indices.count::<u16>() as _);

                let geometry = vk::AccelerationStructureGeometryKHR::builder()
                    .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
                    .geometry(vk::AccelerationStructureGeometryDataKHR {
                        triangles: *triangles,
                    })
                    .flags(vk::GeometryFlagsKHR::OPAQUE);

                let range = vk::AccelerationStructureBuildRangeInfoKHR::builder()
                    .primitive_count((indices.count::<u16>() / 3) as u32);

                let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                    .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                    .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                    .geometries(slice::from_ref(&geometry));

                let build_sizes =
                    AccelerationStructure::build_sizes(self.device(), &geometry_info, &range);
                let scratch_buffer =
                    ScratchBuffer::new(self.device().clone(), build_sizes.build_scratch);

                let acceleration_structure_buffer = Buffer::new(
                    self.device().clone(),
                    build_sizes.acceleration_structure,
                    vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
                    format!("{} acceleration structure", self.name),
                    MemoryLocation::GpuOnly,
                    None,
                );
                // self.buffer = Some(acceleration_structure_buffer);

                let acceleration_structure = AccelerationStructure::new(
                    self.device(),
                    acceleration_structure_buffer,
                    vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
                );

                let geometry_info = geometry_info
                    .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                    .dst_acceleration_structure(unsafe { acceleration_structure.raw() })
                    .scratch_data(vk::DeviceOrHostAddressKHR {
                        device_address: scratch_buffer.device_address(),
                    });

                CommandBuffer::now(self.device(), "build BLAS".to_string(), |cmd| {
                    cmd.build_acceleration_structure(&geometry_info, &range);
                });

                Some(Arc::new(acceleration_structure))
            }
        };
    }
}
