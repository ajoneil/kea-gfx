#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{
    cameras::{Camera, CameraParameters},
    lights::PointLight,
    materials::Material,
    payload::RayPayload,
};
use spirv_std::{
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec3, Vec3A, Vec4},
    ray_tracing::RayFlags,
    Image,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

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
        vertical_field_of_view_radians: 70.0_f32.to_radians(),
        position: vec3(0.0, 1.0, 1.5),
        target_position: vec3(0.0, 1.0, -1.0),
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
                position: vec3(0.0, 1.8, -1.0),
                diffuse: vec3(0.7, 0.7, 0.7),
                specular: Vec3::ZERO,
            };
            let scene_ambience = vec3(0.2, 0.2, 0.2);

            let hit_point = ray.at(distance);
            let light_direction = (scene_light.position - hit_point).normalize();

            let ambient_light = scene_ambience * Vec3::from(payload.material.diffuse);
            let diffuse_dot = light_direction.dot(payload.normal);
            let diffuse_light = if diffuse_dot <= 0.0 {
                Vec3::ZERO
            } else {
                let mut shadow_payload = RayPayload {
                    hit: None,
                    normal: Vec3::ZERO,
                    material: Material {
                        diffuse: Vec3A::ZERO,
                        emit: Vec3A::ZERO,
                    },
                };

                accel_structure.trace_ray(
                    RayFlags::OPAQUE,
                    0xff,
                    0,
                    0,
                    0,
                    hit_point,
                    0.001,
                    light_direction,
                    10000.0,
                    &mut shadow_payload,
                );

                let light_distance = scene_light.position.distance(hit_point);

                if shadow_payload.hit.is_some() && shadow_payload.hit.unwrap() < light_distance {
                    Vec3::ZERO
                } else {
                    let diffuse_light =
                        diffuse_dot * scene_light.diffuse * Vec3::from(payload.material.diffuse);

                    diffuse_light
                }
            };

            let total_light = ambient_light + diffuse_light;
            let white_point = 1.5;
            let white_squared = Vec3::splat(white_point * white_point);
            let tone_mapped =
                (total_light * (1.0 + total_light / white_squared)) / (1.0 + total_light);
            tone_mapped.extend(1.0)
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
