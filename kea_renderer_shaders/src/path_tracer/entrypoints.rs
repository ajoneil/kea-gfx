#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{cameras::Camera, payload::RayPayload};
use spirv_std::{
    glam::{vec2, vec3, vec4, UVec2, UVec3},
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
    let size = vec2(launch_size.x as f32, launch_size.y as f32);
    let pixel_position = vec2(launch_id.x as f32 + 0.5, launch_id.y as f32 + 0.5);
    let ray_target = Camera::new(size.x / size.y).ray_target(vec2(
        pixel_position.x / size.x,
        (size.y - pixel_position.y) / size.y,
    ));

    unsafe {
        accel_structure.trace_ray(
            RayFlags::OPAQUE,
            0xff,
            0,
            0,
            0,
            vec3(0.0, 0.0, 0.0),
            0.01,
            ray_target,
            1000.0,
            payload,
        );

        let output_color = if let Some(depth) = payload.hit {
            const MAX_DEPTH: f32 = 2.0;
            let scaled_depth = 1.0 - (depth / MAX_DEPTH).clamp(0.0, 1.0);
            vec4(scaled_depth, scaled_depth, scaled_depth, 1.0)
        } else {
            vec4(0.0, 0.0, 0.0, 1.0)
        };

        image.write(UVec2::new(launch_id.x, launch_id.y), output_color);
    }
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.hit = None;
}
