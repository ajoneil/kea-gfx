use super::{
    acceleration_structure::AccelerationStructure, scratch_buffer::ScratchBuffer, GeometryInstance,
};
use crate::{commands::CommandBuffer, device::Device, storage::buffers::Buffer};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::{slice, sync::Arc};

pub struct Scene {
    device: Arc<Device>,
    name: String,
    instances: Vec<GeometryInstance>,
    acceleration_structure: Option<Arc<AccelerationStructure>>,
    instances_buffer: Option<Buffer>,
}

impl Scene {
    pub fn new(device: Arc<Device>, name: String) -> Self {
        Self {
            device,
            name,
            instances: vec![],
            acceleration_structure: None,
            instances_buffer: None,
        }
    }

    pub fn instances(&self) -> &[GeometryInstance] {
        &self.instances
    }

    pub fn add_instance(&mut self, instance: GeometryInstance) {
        self.instances.push(instance);
    }

    pub fn acceleration_structure(&self) -> &Arc<AccelerationStructure> {
        if self.acceleration_structure.is_none() {
            panic!("Scene {} isn't built", self.name);
        }

        self.acceleration_structure.as_ref().unwrap()
    }

    pub fn build(&mut self) {
        let instances: Vec<vk::AccelerationStructureInstanceKHR> = self
            .instances
            .iter()
            .map(|instance| unsafe { instance.raw() })
            .collect();

        let instances_buffer = Buffer::new_from_data(
            self.device.clone(),
            &instances,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            "scene build".to_string(),
            MemoryLocation::GpuOnly,
        );

        let geometry_data = vk::AccelerationStructureGeometryDataKHR {
            instances: vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                .data(vk::DeviceOrHostAddressConstKHR {
                    device_address: instances_buffer.device_address(),
                })
                .array_of_pointers(false)
                .build(),
        };

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .geometry(geometry_data)
            .build();

        let range = vk::AccelerationStructureBuildRangeInfoKHR {
            primitive_count: instances.len() as _,
            primitive_offset: 0,
            first_vertex: 0,
            transform_offset: 0,
        };

        let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .geometries(slice::from_ref(&geometry))
            .build();

        let build_sizes = AccelerationStructure::build_sizes(&self.device, geometry_info, range);
        let scratch_buffer = ScratchBuffer::new(self.device.clone(), build_sizes.build_scratch);

        let acceleration_structure_buffer = Buffer::new(
            self.device.clone(),
            build_sizes.acceleration_structure,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            format!("{} acceleration structure", self.name),
            MemoryLocation::GpuOnly,
        );
        let acceleration_structure = AccelerationStructure::new(
            &self.device,
            acceleration_structure_buffer,
            vk::AccelerationStructureTypeKHR::TOP_LEVEL,
        );

        let geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .geometries(slice::from_ref(&geometry))
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .dst_acceleration_structure(unsafe { acceleration_structure.raw() })
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: scratch_buffer.device_address(),
            })
            .build();

        CommandBuffer::now(&self.device, |cmd| {
            cmd.build_acceleration_structure(geometry_info, range);
        });

        self.instances_buffer = Some(instances_buffer);
        self.acceleration_structure = Some(Arc::new(acceleration_structure));
    }
}
