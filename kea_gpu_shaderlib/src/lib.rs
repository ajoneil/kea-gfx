#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// #![deny(warnings)]

use spirv_std::glam::Vec3;

#[repr(C)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
