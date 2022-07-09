use kea_gpu_shaderlib::Ray;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{
    arch::report_intersection,
    glam::{vec3, Vec3},
};

use crate::{materials::Material, payload::RayPayload, spheres::Sphere};

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

    ray_payload.material = match sphere_id % 4 {
        1 => Material {
            ambient: vec3(1.0, 0.0, 0.0),
            diffuse: vec3(1.0, 0.0, 0.0),
            specular: vec3(0.7, 0.5, 0.5),
            shininess: 128.0,
        },
        2 => Material {
            ambient: vec3(0.0, 1.0, 0.0),
            diffuse: vec3(0.0, 1.0, 0.0),
            specular: vec3(0.4, 0.5, 0.4),
            shininess: 10.0,
        },
        3 => Material {
            ambient: vec3(0.0, 0.0, 1.0),
            diffuse: vec3(0.0, 0.0, 1.0),
            specular: vec3(0.5, 0.5, 0.7),
            shininess: 5.0,
        },
        _ => Material {
            ambient: vec3(0.7, 0.7, 0.7),
            diffuse: vec3(0.7, 0.7, 0.7),
            specular: Vec3::ZERO,
            shininess: 0.0,
        },
    }
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
