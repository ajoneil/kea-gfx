#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{
    glam::{vec4, UVec2, UVec3, Vec2, Vec3, Vec4},
    ray_tracing::AccelerationStructure,
    Image,
};

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

#[spirv(ray_generation)]
pub fn generate_rays(
    #[spirv(launch_id)] launch_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] _accel_structure: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &mut Image!(2D, format=rgba32f, sampled=false),
) {
    unsafe {
        image.write(
            UVec2::new(launch_id.x, launch_id.y),
            vec4(0.5, 0.5, 0.5, 1.0),
        );
    }
}

#[spirv(miss)]
pub fn ray_miss() {}

#[spirv(closest_hit)]
pub fn ray_hit() {}

#[spirv(intersection)]
pub fn intersect_sphere() {}
