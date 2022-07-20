use kea_gpu_shaderlib::Ray;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{
    cameras::{Camera, CameraParameters},
    payload::RayPayload,
};
use spirv_std::{
    glam::{vec2, vec3, DVec3, Quat, UVec2, UVec3, Vec3, Vec4, Vec4Swizzles},
    ray_tracing::RayFlags,
    Image,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use super::rand::Random;

#[repr(C)]
pub struct PushConstants {
    pub iteration: u64,
}

#[spirv(ray_generation)]
pub fn generate_rays(
    #[spirv(launch_id)] launch_id: UVec3,
    #[spirv(launch_size)] launch_size: UVec3,
    #[spirv(ray_payload)] payload: &mut RayPayload,
    #[spirv(descriptor_set = 0, binding = 0)]
    accel_structure: &spirv_std::ray_tracing::AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] output_image: &mut Image!(2D, format=rgba32f, sampled=false),
    #[spirv(descriptor_set = 0, binding = 6)] light_image: &mut Image!(2D, format=rgba32f, sampled=false),
    #[spirv(push_constant)] constants: &PushConstants,
) {
    let size = vec2(launch_size.x as f32, launch_size.y as f32);

    let mut rand = Random::new(
        launch_id.x * launch_size.x,
        launch_id.y * launch_size.y,
        payload.hit.unwrap_or(launch_id.y as f32 * 42.0) as u32,
        constants.iteration as u32,
    );

    let mut total_light = DVec3::ZERO;

    let num_samples = 10;

    for _ in 0..num_samples {
        let pixel_position = vec2(
            launch_id.x as f32 + rand.next_float() - 0.5,
            launch_id.y as f32 + rand.next_float() - 0.5,
        );

        let mut ray = Camera::new(CameraParameters {
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

        let mut sample_light = DVec3::ZERO;
        let mut remaining_bounces = 15;
        let mut contribution = Vec3::ONE;

        while remaining_bounces > 0 {
            remaining_bounces = remaining_bounces - 1;

            unsafe {
                accel_structure.trace_ray(
                    RayFlags::OPAQUE,
                    0xff,
                    0,
                    0,
                    0,
                    ray.origin,
                    0.01,
                    ray.direction,
                    10000.0,
                    payload,
                );
            }

            if let Some(distance) = payload.hit {
                sample_light +=
                    Vec3::from(payload.material.emit).as_dvec3() * contribution.as_dvec3();

                if remaining_bounces > 0 {
                    let origin = ray.at(distance);

                    // let mut u: f32;
                    // let mut v: f32;
                    // loop {
                    //     u = rand.next_float() * 2.0 - 1.0;
                    //     v = rand.next_float() * 2.0 - 1.0;

                    //     if (u * u + v * v) < 1.0 {
                    //         break;
                    //     }
                    // }
                    // let w = (1.0 - u * u - v * v).sqrt();

                    // let direction = (Vec3::X * u + Vec3::Y * v + payload.normal * w).normalize();
                    // let radial = rand.next_float().sqrt();
                    // let r = rand.next_float();
                    // let theta = 2.0 * core::f32::consts::PI * r;

                    // let x = radial * theta.cos();
                    // let y = radial * theta.sin();

                    // let direction = vec3(x, y, (1.0 - r).max(0.0).sqrt()).normalize();
                    // let direction =
                    //     Quat::from_rotation_arc(Vec3::Z, payload.normal).mul_vec3(direction);
                    let random_angle = rand.next_float();
                    let radial = random_angle.sqrt();
                    let theta = 2.0 * core::f32::consts::PI * rand.next_float();

                    let x = radial * theta.cos();
                    let y = radial * theta.sin();

                    let direction = vec3(x, y, (1.0 - random_angle).max(0.0).sqrt()).normalize();
                    let direction = Quat::from_rotation_arc(Vec3::Z, payload.normal)
                        .mul_vec3(direction)
                        .normalize();

                    // let mut direction;
                    // loop {
                    //     direction = vec3(rand.next_float(), rand.next_float(), rand.next_float())
                    //         .normalize();

                    //     if payload.normal.dot(direction) > 0.0 {
                    //         break;
                    //     }
                    // }

                    contribution *= Vec3::from(payload.material.diffuse) * core::f32::consts::PI;

                    ray = Ray { origin, direction }
                }
            } else {
                remaining_bounces = 0;
            }
        }

        total_light += sample_light;
    }

    total_light = total_light / num_samples as f64;

    let new_total = if constants.iteration > 0 {
        let existing: Vec4 = light_image.read(UVec2::new(launch_id.x, launch_id.y));
        existing.xyz().as_dvec3() * (1.0 - 1.0 / constants.iteration as f64)
            + total_light * (1.0 / constants.iteration as f64)
    } else {
        total_light
    };

    let white_point = 2.0;
    let white_squared = Vec3::splat(white_point * white_point);
    let tone_mapped = (new_total.as_vec3() * (1.0 + new_total.as_vec3() / white_squared))
        / (1.0 + new_total.as_vec3());

    unsafe {
        light_image.write(
            UVec2::new(launch_id.x, launch_id.y),
            new_total.as_vec3().extend(1.0),
        );
        output_image.write(
            UVec2::new(launch_id.x, launch_id.y),
            tone_mapped.extend(1.0),
        );
    }
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.hit = None;
}
