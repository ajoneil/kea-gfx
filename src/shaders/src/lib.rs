#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{vec4, Vec2, Vec3, Vec4};

#[repr(C)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}

#[spirv(vertex)]
pub fn main_vertex(
    position: Vec2,
    color: Vec3,
    #[spirv(position)] out_pos: &mut Vec4,
    fragment_color: &mut Vec3,
) {
    *out_pos = vec4(position.x, position.y, 0.0, 1.0);
    *fragment_color = color;
}

#[spirv(fragment)]
pub fn main_fragment(fragment_color: Vec3, output: &mut Vec4) {
    *output = vec4(fragment_color.x, fragment_color.y, fragment_color.z, 1.0);
}
