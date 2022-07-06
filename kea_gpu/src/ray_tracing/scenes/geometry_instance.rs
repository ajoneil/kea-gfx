use super::Geometry;
use ash::vk;
use std::sync::Arc;

pub struct GeometryInstance {
    transform: vk::TransformMatrixKHR,
    hit_group: u32,
    geometry: Arc<Geometry>,
}

impl GeometryInstance {
    pub fn new(geometry: Arc<Geometry>, hit_group: u32) -> Self {
        let identity = vk::TransformMatrixKHR {
            matrix: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        };

        Self {
            transform: identity,
            hit_group,
            geometry,
        }
    }

    pub fn geometry(&self) -> &Arc<Geometry> {
        &self.geometry
    }

    pub unsafe fn raw(&self) -> vk::AccelerationStructureInstanceKHR {
        let custom_index = 0;
        let mask = 0xff;
        let flags = vk::GeometryInstanceFlagsKHR::FORCE_OPAQUE
            .as_raw()
            .try_into()
            .unwrap();

        vk::AccelerationStructureInstanceKHR {
            transform: self.transform,
            instance_custom_index_and_mask: vk::Packed24_8::new(custom_index, mask),
            instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                self.hit_group,
                flags,
            ),
            acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                device_handle: self.geometry().acceleration_structure().device_address(),
                //host_handle: self.geometry.acceleration_structure().raw(),
            },
        }
    }
}
