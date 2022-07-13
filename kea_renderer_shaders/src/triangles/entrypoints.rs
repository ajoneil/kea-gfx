use spirv_std::glam::{vec3, Vec3};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{materials::Material, payload::RayPayload};

#[spirv(closest_hit)]
pub fn triangle_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
) {
    ray_payload.hit = Some(hit_max);
    ray_payload.material = Material {
        ambient: vec3(0.5, 0.5, 0.5),
        diffuse: vec3(0.4, 0.4, 0.4),
        specular: Vec3::ZERO,
        shininess: 0.0,
    }
}
