#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// #![deny(warnings)]
#![feature(const_type_id)]

use spirv_std::glam::Vec3;

pub mod slots;

#[repr(C)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
