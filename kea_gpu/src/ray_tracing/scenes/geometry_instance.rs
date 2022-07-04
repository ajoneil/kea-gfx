use super::Geometry;
use ash::vk;
use std::sync::Arc;

pub struct GeometryInstance {
    transform: vk::TransformMatrixKHR,
    geometry: Arc<Geometry>,
}

impl GeometryInstance {
    pub fn new(geometry: Arc<Geometry>) -> Self {
        let identity = vk::TransformMatrixKHR {
            matrix: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        };

        Self {
            transform: identity,
            geometry,
        }
    }

    pub fn geometry(&self) -> &Arc<Geometry> {
        &self.geometry
    }

    pub unsafe fn raw(&self) -> vk::AccelerationStructureInstanceKHR {
        let custom_index = 0;
        let mask = 0xff;
        let shader_binding_table_record_offset = 0;
        let flags = vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE
            .as_raw()
            .try_into()
            .unwrap();

        vk::AccelerationStructureInstanceKHR {
            transform: self.transform,
            instance_custom_index_and_mask: vk::Packed24_8::new(custom_index, mask),
            instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                shader_binding_table_record_offset,
                flags,
            ),
            acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                device_handle: self.geometry().acceleration_structure().device_address(),
                //host_handle: self.geometry.acceleration_structure().raw(),
            },
        }
    }
}
