#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::payload::{HitType, RayPayload};
use spirv_std::{
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec2, Vec3},
    ray_tracing::RayFlags,
    Image,
};

#[spirv(ray_generation)]
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
            RayFlags::OPAQUE,
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

        let output_color = if payload.hit_type == HitType::Hit {
            vec4(0.5, 0.5, 0.5, 1.0)
        } else {
            vec4(0.0, 0.0, 0.0, 0.0)
        };

        image.write(UVec2::new(launch_id.x, launch_id.y), output_color);
    }
}

pub fn ray_for_pixel(pixel_position: Vec2, size: Vec2) -> Vec3 {
    let aspect_ratio = size.x / size.y;
    let uv = vec2(
        pixel_position.x / size.x,
        (size.y - pixel_position.y) / size.y,
    );
    let direction = uv * 2.0 - 1.0;
    let target = vec3(direction.x * aspect_ratio, direction.y, -1.0);
    target.normalize()
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.hit_type = HitType::Miss;
}
