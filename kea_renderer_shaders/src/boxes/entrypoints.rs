use kea_gpu_shaderlib::Ray;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::payload::RayPayload;
use spirv_std::{arch::report_intersection, glam::Vec3};

use super::Boxo;

#[spirv(closest_hit)]
pub fn hit_box(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
    #[spirv(primitive_id)] box_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] boxes: &mut [Boxo],
) {
    let boxo = boxes[box_id];
    ray_payload.hit = Some(hit_max);

    ray_payload.normal = boxo.normal(Ray {
        origin: ray_origin,
        direction: ray_direction,
    });

    ray_payload.material = boxo.material();
}

#[spirv(intersection)]
pub fn intersect_box(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(primitive_id)] box_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] boxes: &mut [Boxo],
) {
    let boxo = boxes[box_id];

    if let Some(hit) = boxo.intersect_ray(Ray {
        origin: ray_origin,
        direction: ray_direction,
    }) {
        unsafe {
            report_intersection(hit, 0);
        }
    }
}
