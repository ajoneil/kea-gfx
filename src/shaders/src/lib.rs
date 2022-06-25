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
    arch::report_intersection,
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec2, Vec3, Vec4},
    num_traits::Float,
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
    #[spirv(ray_payload)] payload: &mut RayPayload,
    #[spirv(descriptor_set = 0, binding = 0)] accel_structure: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &mut Image!(2D, format=rgba32f, sampled=false),
) {
    let ray_direction = ray_for_pixel(launch_id.x, launch_id.y);

    unsafe {
        accel_structure.trace_ray(
            RayFlags::NONE,
            0xff,
            0,
            0,
            0,
            vec3(0.0, 0.0, 0.0),
            0.01,
            ray_direction,
            1000.0,
            payload,
        );

        image.write(
            UVec2::new(launch_id.x, launch_id.y),
            vec4(payload.color.x, payload.color.y, payload.color.z, 1.0),
        );
    }
}

pub fn ray_for_pixel(x: u32, y: u32) -> Vec3 {
    let pixel_center = vec2(x as f32 + 0.5, y as f32 + 0.5);
    let uv = pixel_center / vec2(1920.0, 1080.0);
    let direction = uv * 2.0 - 1.0;
    let target = vec3(direction.x, direction.y, 1.0);
    target.normalize()
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(0.0, 0.0, 1.0);
}

#[spirv(closest_hit)]
pub fn ray_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(1.0, 0.0, 0.0);
}

#[repr(C)]
pub struct Sphere {
    position: Vec3,
    radius: f32,
}

#[spirv(intersection)]
pub fn intersect_sphere(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(primitive_id)] sphere_id: u32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] spheres: &mut [Sphere],
) {
    // unsafe {
    //     report_intersection(1.0, 4);
    // }

    let sphere = &spheres[sphere_id as usize];

    let oc = ray_origin - sphere.position;
    let a = ray_direction.dot(ray_direction);
    let b = 2.0 * oc.dot(ray_direction);
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant >= 0.0 {
        let hit = (-b - discriminant.sqrt()) / (2.0 * a);
        unsafe {
            report_intersection(hit, 4);
        }
    }
}
