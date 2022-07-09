use spirv_std::glam::Vec3;

#[repr(C)]
pub struct Material {
    pub ambient_color: Vec3,
    pub diffuse_color: Vec3,
}
