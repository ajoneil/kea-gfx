use spirv_std::glam::Vec3;

#[repr(C)]
pub struct RayPayload {
    pub color: Vec3,
}
