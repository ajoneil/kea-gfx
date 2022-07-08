#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::payload::{HitType, RayPayload};

#[spirv(closest_hit)]
pub fn triangle_hit(#[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload) {
    ray_payload.hit_type = HitType::Hit;
}
