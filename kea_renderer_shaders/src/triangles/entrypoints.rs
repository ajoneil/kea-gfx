#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::{materials::Material, payload::RayPayload};
use spirv_std::glam::{vec3a, Vec3A};

#[spirv(closest_hit)]
pub fn triangle_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
) {
    ray_payload.hit = Some(hit_max);
    ray_payload.material = Material {
        diffuse: vec3a(0.4, 0.4, 0.4),
        emit: Vec3A::ZERO,
    }
}
