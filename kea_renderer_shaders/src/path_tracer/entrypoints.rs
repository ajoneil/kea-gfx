#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{
    cameras::{Camera, CameraParameters},
    lights::PointLight,
    materials::Material,
    payload::RayPayload,
};
use spirv_std::{
    glam::{vec2, vec3, vec4, UVec2, UVec3, Vec3, Vec4},
    num_traits::Pow,
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
        vertical_field_of_view_radians: 60.0_f32.to_radians(),
        position: vec3(0.0, 1.5, 0.0),
        target_position: vec3(0.0, 0.8, -1.5),
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
                position: vec3(2.5, 4.0, 0.0),
                diffuse: vec3(0.7, 0.7, 0.7),
                specular: vec3(0.6, 0.6, 0.6),
            };
            let scene_ambience = vec3(0.2, 0.2, 0.2);

            let hit_point = ray.at(distance);
            let light_direction = (scene_light.position - hit_point).normalize();

            let ambient_light = scene_ambience * payload.material.ambient;
            let diffuse_dot = light_direction.dot(payload.normal);
            let (diffuse_light, specular_light) = if diffuse_dot <= 0.0 {
                (Vec3::ZERO, Vec3::ZERO)
            } else {
                const SHADOWS: bool = true;

                let in_shadow = if !SHADOWS {
                    false
                } else {
                    let mut shadow_payload = RayPayload {
                        hit: None,
                        normal: Vec3::ZERO,
                        material: Material {
                            ambient: Vec3::ZERO,
                            diffuse: Vec3::ZERO,
                            specular: Vec3::ZERO,
                            shininess: 0.0,
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
                    shadow_payload.hit.is_some() && shadow_payload.hit.unwrap() < light_distance
                };

                if in_shadow {
                    (Vec3::ZERO, Vec3::ZERO)
                } else {
                    let diffuse_light =
                        diffuse_dot * scene_light.diffuse * payload.material.diffuse;
                    let specular_light = {
                        let reflection_direction =
                            2.0 * light_direction.dot(payload.normal) * payload.normal
                                - light_direction;
                        let viewer_direction = ray.direction * -1.0;

                        viewer_direction
                            .dot(reflection_direction)
                            .pow(payload.material.shininess)
                            * scene_light.specular
                            * payload.material.specular
                    };

                    (diffuse_light, specular_light.max(Vec3::ZERO))
                }
            };

            ((ambient_light + diffuse_light + specular_light) * 0.7).extend(1.0)
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
