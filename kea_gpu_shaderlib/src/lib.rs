#![cfg_attr(target_arch = "spirv", no_std)]
// #![deny(warnings)]

use spirv_std::glam::Vec3;

pub mod shaders;
pub mod slots;

mod ray;
pub use ray::Ray;

#[repr(C)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
