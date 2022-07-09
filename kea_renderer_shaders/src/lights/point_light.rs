use spirv_std::glam::Vec3;

#[repr(C)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: Vec3,
}
