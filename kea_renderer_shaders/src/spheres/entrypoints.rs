#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{
    arch::report_intersection,
    glam::{vec3, Vec3},
};

// Needed for .sqrt()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::{payload::RayPayload, spheres::Sphere};

#[spirv(closest_hit)]
pub fn sphere_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(1.0, 0.0, 0.0);
}

#[spirv(intersection)]
pub fn intersect_sphere(
    #[spirv(world_ray_origin)] ray_origin: Vec3,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(primitive_id)] sphere_id: usize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] spheres: &mut [Sphere],
) {
    // unsafe {
    //     if cfg!(target_arch = "spirv") {
    //         debug_printfln!("id: %d", sphere_id as i32);
    //     }
    // }

    let sphere = spheres[sphere_id];
    let oc = ray_origin - sphere.position();
    let a = ray_direction.dot(ray_direction);
    let b = 2.0 * oc.dot(ray_direction);
    let c = oc.dot(oc) - (sphere.radius() * sphere.radius());
    let discriminant = b * b - (4.0 * a * c);

    if discriminant >= 0.0 {
        let hit = (-b - discriminant.sqrt()) / (2.0 * a);
        unsafe {
            report_intersection(hit, 4);
        }
    }
}
