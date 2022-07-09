use crate::materials::Material;
use spirv_std::glam::Vec3;

#[repr(C)]
pub struct RayPayload {
    pub hit: Option<f32>,
    pub normal: Vec3,
    pub material: Material,
}
