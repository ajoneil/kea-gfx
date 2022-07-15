use spirv_std::glam::Vec3A;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Material {
    pub diffuse: Vec3A,
    pub emit: Vec3A,
}
