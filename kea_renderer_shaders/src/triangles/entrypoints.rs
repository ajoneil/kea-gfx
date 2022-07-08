#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::payload::RayPayload;
use spirv_std::glam::vec3;

#[spirv(closest_hit)]
pub fn triangle_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.color = vec3(0.0, 1.0, 0.0);
}
