#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{
    cameras::{Camera, CameraParameters},
    lights::PointLight,
    payload::RayPayload,
};
use spirv_std::{
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec4},
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
    let ray = Camera::new(CameraParameters {
        aspect_ratio: size.x / size.y,
        vertical_field_of_view_radians: 80.0_f32.to_radians(),
        position: vec3(0.0, 1.0, 0.0),
        target_position: vec3(0.0, 0.4, -1.5),
        ..Default::default()
    })
    .ray(
        pixel_position.x / size.x,
        (size.y - pixel_position.y) / size.y,
    );

    unsafe {
        accel_structure.trace_ray(
            RayFlags::OPAQUE,
            0xff,
            0,
            0,
            0,
            ray.origin,
            0.001,
            ray.direction,
            10000.0,
            payload,
        );

        let output_color: Vec4 = if let Some(distance) = payload.hit {
            let scene_light = PointLight {
                position: vec3(1.0, 5.0, -0.5),
                intensity: vec3(1.0, 1.0, 1.0),
            };
            let scene_ambience = vec3(0.1, 0.1, 0.1);

            let ambient_light = scene_ambience * payload.material.ambient_color;
            let diffuse_light = {
                let hit_point = ray.at(distance);
                let light_direction = (scene_light.position - hit_point).normalize();
                let light_amount = light_direction.dot(payload.normal);
                light_amount * scene_light.intensity * payload.material.diffuse_color
            };

            Vec4::from((ambient_light + diffuse_light, 1.0))
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
