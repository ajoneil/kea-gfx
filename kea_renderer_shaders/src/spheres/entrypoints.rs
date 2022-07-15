#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{payload::RayPayload, spheres::Sphere};
use kea_gpu_shaderlib::Ray;
use spirv_std::{arch::report_intersection, glam::Vec3};

#[spirv(closest_hit)]
pub fn sphere_hit(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
    #[spirv(primitive_id)] sphere_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] spheres: &mut [Sphere],
) {
    let sphere = spheres[sphere_id];
    ray_payload.hit = Some(hit_max);
    ray_payload.normal = sphere.normal(Ray {
        origin: ray_origin,
        direction: ray_direction,
    });

    ray_payload.material = sphere.material();
}

#[spirv(intersection)]
pub fn intersect_sphere(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(primitive_id)] sphere_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] spheres: &mut [Sphere],
) {
    let sphere = spheres[sphere_id];

    if let Some(hit) = sphere.intersect_ray(Ray {
        origin: ray_origin,
        direction: ray_direction,
    }) {
        unsafe {
            report_intersection(hit, 0);
        }
    }
}
