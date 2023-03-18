use super::Geometry;
use ash::vk;
use spirv_std::glam::Affine3A;
use std::sync::Arc;

pub struct GeometryInstance {
    transform: vk::TransformMatrixKHR,
    hit_group: u32,
    geometry: Arc<Geometry>,
    custom_index: u32,
}

impl GeometryInstance {
    pub fn new(
        geometry: Arc<Geometry>,
        hit_group: u32,
        transform: Affine3A,
        custom_index: u32,
    ) -> Self {
        let transform = vk::TransformMatrixKHR {
            matrix: [
                transform.x_axis.x,
                transform.y_axis.x,
                transform.z_axis.x,
                transform.translation.x,
                transform.x_axis.y,
                transform.y_axis.y,
                transform.z_axis.y,
                transform.translation.y,
                transform.x_axis.z,
                transform.y_axis.z,
                transform.z_axis.z,
                transform.translation.z,
            ],
        };

        Self {
            transform,
            hit_group,
            geometry,
            custom_index,
        }
    }

    pub fn geometry(&self) -> &Arc<Geometry> {
        &self.geometry
    }

    pub unsafe fn raw(&self) -> vk::AccelerationStructureInstanceKHR {
        let flags = vk::GeometryInstanceFlagsKHR::FORCE_OPAQUE
            .as_raw()
            .try_into()
            .unwrap();

        vk::AccelerationStructureInstanceKHR {
            transform: self.transform,
            instance_custom_index_and_mask: vk::Packed24_8::new(self.custom_index, 0xff),
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
