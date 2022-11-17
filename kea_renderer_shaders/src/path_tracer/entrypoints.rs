use kea_gpu_shaderlib::Ray;
use spirv_std::spirv;

use crate::{
    cameras::{Camera, CameraParameters},
    payload::RayPayload,
};
use spirv_std::{
    glam::{vec2, vec3, Quat, UVec2, UVec3, Vec2, Vec3, Vec4, Vec4Swizzles},
    ray_tracing::RayFlags,
    Image,
};

use super::rand::Random;

#[repr(C)]
pub struct PushConstants {
    pub iteration: u64,
}

const NUM_SAMPLES: u32 = 5;
const NUM_BOUNCES: u32 = 15;
const WHITE_POINT: f32 = 2.0;

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
    let pixel_position = vec2(launch_id.x as f32, launch_id.y as f32);

    let camera = Camera::new(CameraParameters {
        aspect_ratio: size.x / size.y,
        vertical_field_of_view_radians: 70.0_f32.to_radians(),
        position: vec3(0.0, 1.0, 1.5),
        target_position: vec3(0.0, 1.0, -1.0),
        ..Default::default()
    });

    let mut rand = Random::new(
        launch_id.x * launch_size.x,
        launch_id.y * launch_size.y,
        payload.hit.unwrap_or(launch_id.y as f32 * 42.0) as u32,
        constants.iteration as u32,
    );

    let iteration_light = multisample_pixel(
        accel_structure,
        payload,
        &camera,
        size,
        pixel_position,
        &mut rand,
        NUM_SAMPLES,
    );

    let total_light = update_light_total(
        UVec2::new(launch_id.x, launch_id.y),
        light_image,
        constants.iteration,
        iteration_light,
    );

    unsafe {
        output_image.write(
            UVec2::new(launch_id.x, launch_id.y),
            tone_map(total_light, WHITE_POINT).extend(1.0),
        );
    }
}

fn multisample_pixel(
    accel_structure: &spirv_std::ray_tracing::AccelerationStructure,
    payload: &mut RayPayload,
    camera: &Camera,
    size: Vec2,
    pixel_position: Vec2,
    rand: &mut Random,
    num_samples: u32,
) -> Vec3 {
    let mut accumulated_light = Vec3::ZERO;
    for _ in 0..num_samples {
        accumulated_light += sample_pixel(
            accel_structure,
            payload,
            camera,
            size,
            jittered_position(pixel_position, rand),
            rand,
        )
    }

    accumulated_light / num_samples as f32
}

fn jittered_position(position: Vec2, rand: &mut Random) -> Vec2 {
    vec2(
        position.x + rand.next_float() - 0.5,
        position.y + rand.next_float() - 0.5,
    )
}

fn tone_map(light: Vec3, white_point: f32) -> Vec3 {
    (light * (1.0 + light / Vec3::splat(white_point * white_point))) / (1.0 + light)
}

fn update_light_total(
    pixel_position: UVec2,
    light_image: &mut Image!(2D, format=rgba32f, sampled=false),
    iteration: u64,
    iteration_light: Vec3,
) -> Vec3 {
    let total_light = if iteration > 0 {
        let existing: Vec4 = light_image.read(pixel_position);
        existing.xyz() * (1.0 - 1.0 / iteration as f32) + iteration_light * (1.0 / iteration as f32)
    } else {
        iteration_light
    };

    unsafe {
        light_image.write(pixel_position, total_light.extend(1.0));
    }

    total_light
}

fn sample_pixel(
    accel_structure: &spirv_std::ray_tracing::AccelerationStructure,
    payload: &mut RayPayload,
    camera: &Camera,
    size: Vec2,
    pixel_position: Vec2,
    rand: &mut Random,
) -> Vec3 {
    let mut light = Vec3::ZERO;

    let mut ray = camera.ray(
        pixel_position.x / size.x,
        (size.y as f32 - pixel_position.y) / size.y,
    );

    let mut contribution = Vec3::ONE;

    for _ in 0..NUM_BOUNCES {
        let BounceSample {
            hit,
            light_emitted,
            next_ray,
            next_contribution,
        } = sample_bounce(accel_structure, ray, payload, rand);

        if hit {
            light += light_emitted * contribution;
            ray = next_ray;
            contribution *= next_contribution;
        }

        if !hit || contribution.max_element() < 0.001 {
            break;
        }
    }

    light
}

struct BounceSample {
    hit: bool,
    light_emitted: Vec3,
    next_ray: Ray,
    next_contribution: Vec3,
}

fn sample_bounce(
    accel_structure: &spirv_std::ray_tracing::AccelerationStructure,
    ray: Ray,
    payload: &mut RayPayload,
    rand: &mut Random,
) -> BounceSample {
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
    }

    if let Some(distance) = payload.hit {
        let light_emitted = Vec3::from(payload.material.emit);

        let direction = Quat::from_rotation_arc(Vec3::Z, payload.normal)
            .mul_vec3(rand.random_hemisphere_direction())
            .normalize();

        let next_ray = Ray {
            origin: ray.at(distance),
            direction,
        };
        let next_contribution = Vec3::from(payload.material.diffuse) * core::f32::consts::PI;

        BounceSample {
            hit: true,
            light_emitted,
            next_ray,
            next_contribution,
        }
    } else {
        BounceSample {
            hit: false,
            light_emitted: Vec3::ZERO,
            next_ray: Ray {
                origin: Vec3::ZERO,
                direction: Vec3::ZERO,
            },
            next_contribution: Vec3::ZERO,
        }
    }
}

#[spirv(miss)]
pub fn ray_miss(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.hit = None;
}
