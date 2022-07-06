#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::{
    arch::report_intersection,
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec2, Vec3},
    macros::debug_printfln,
    Image,
};

// Needed for .sqrt()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::Sphere;

#[repr(C)]
pub struct RayPayload {
    color: Vec3,
}

#[spirv(ray_generation)]
#[rustfmt::skip]
pub fn generate_rays(
    #[spirv(launch_id)] launch_id: UVec3,
    #[spirv(launch_size)] launch_size: UVec3,
    #[spirv(ray_payload)] payload: &mut RayPayload,
    #[spirv(descriptor_set = 0, binding = 0)]
    accel_structure: &spirv_std::ray_tracing::AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &mut Image!(2D, format=rgba32f, sampled=false),
) {
    let ray_direction = ray_for_pixel(
        vec2(launch_id.x as f32 + 0.5, launch_id.y as f32 + 0.5),
        vec2(launch_size.x as f32, launch_size.y as f32),
    );

    unsafe {
        accel_structure.trace_ray(
            spirv_std::ray_tracing::RayFlags::NONE,
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

pub fn ray_for_pixel(pixel_position: Vec2, size: Vec2) -> Vec3 {
    let aspect_ratio = size.x / size.y;
    let uv = pixel_position / size;
    let direction = uv * 2.0 - 1.0;
    let target = vec3(direction.x * aspect_ratio, direction.y, 1.0);
    target.normalize()
}

#[spirv(miss)]
#[rustfmt::skip]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(0.0, 0.0, 1.0);
}

#[spirv(closest_hit)]
#[rustfmt::skip]
pub fn triangle_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(0.0, 1.0, 0.0);
}

#[spirv(closest_hit)]
#[rustfmt::skip]
pub fn sphere_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(1.0, 0.0, 0.0);
}

#[spirv(intersection)]
#[rustfmt::skip]
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
