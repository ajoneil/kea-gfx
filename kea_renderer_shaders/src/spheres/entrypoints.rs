#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{arch::report_intersection, glam::Vec3};

use crate::{payload::RayPayload, spheres::Sphere};

#[spirv(closest_hit)]
pub fn sphere_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
) {
    ray_payload.hit = Some(hit_max);
}

#[spirv(intersection)]
pub fn intersect_sphere(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(primitive_id)] sphere_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] spheres: &mut [Sphere],
) {
    let sphere = spheres[sphere_id];

    if let Some(hit) = sphere.intersect_ray(ray_origin, ray_direction) {
        unsafe {
            report_intersection(hit, 4);
        }
    }
}