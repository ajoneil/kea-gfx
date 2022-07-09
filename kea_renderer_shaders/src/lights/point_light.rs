use spirv_std::glam::Vec3;

#[repr(C)]
pub struct PointLight {
    pub position: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}
