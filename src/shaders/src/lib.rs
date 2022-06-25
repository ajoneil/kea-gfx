#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// #![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{
    glam::{vec3, vec4, UVec2, UVec3, Vec2, Vec3, Vec4},
    ray_tracing::{AccelerationStructure, RayFlags},
    Image,
};

#[repr(C)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}

#[repr(C)]
pub struct RayPayload {
    color: Vec3,
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
    #[spirv(descriptor_set = 0, binding = 0)] accel_structure: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &mut Image!(2D, format=rgba32f, sampled=false),
) {
    let ray_direction = ray_for_pixel(launch_id.x, launch_id.y);
    let mut payload = RayPayload {
        color: vec3(0.0, 0.0, 0.0),
    };

    unsafe {
        // accel_structure.trace_ray(
        //     RayFlags::OPAQUE,
        //     0xff,
        //     0,
        //     0,
        //     0,
        //     vec3(0.0, 0.0, 0.0),
        //     0.001,
        //     ray_direction,
        //     10000.0,
        //     &mut payload,
        // );

        image.write(
            UVec2::new(launch_id.x, launch_id.y),
            vec4(payload.color.x, payload.color.y, payload.color.z, 1.0),
        );
    }
}

pub fn ray_for_pixel(x: u32, y: u32) -> Vec3 {
    let width = 1920;
    let height = 1080;
    let aspect_ratio = width as f32 / height as f32;

    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let u = x as f32 / width as f32;
    let v = y as f32 / height as f32;
    vec3(u * viewport_width, v * viewport_height, focal_length)
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(0.0, 0.0, 1.0);
}

#[spirv(closest_hit)]
pub fn ray_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(1.0, 0.0, 0.0);
}

#[spirv(intersection)]
pub fn intersect_sphere() {}
