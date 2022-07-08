#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use crate::payload::RayPayload;

#[spirv(closest_hit)]
pub fn triangle_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
) {
    ray_payload.hit = Some(hit_max);
}
