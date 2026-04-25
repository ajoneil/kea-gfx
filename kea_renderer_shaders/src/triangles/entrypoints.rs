use spirv_std::spirv;

use super::Mesh;
use crate::payload::RayPayload;
use spirv_std::glam::{vec3, Vec3};

#[spirv(closest_hit)]
pub fn triangle_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] meshes: &[Mesh],
    #[spirv(hit_triangle_vertex_positions)] points: [Vec3; 3],
    #[spirv(instance_custom_index)] mesh_id: usize,
) {
    let u = points[1] - points[0];
    let v = points[2] - points[0];
    let mut normal = vec3(
        u.y * v.z - u.z * v.y,
        u.z * v.x - u.x * v.z,
        u.x * v.y - u.y * v.x,
    );

    if ray_direction.dot(normal) > 0.0 {
        normal = normal * -1.0;
    }

    *ray_payload = RayPayload {
        hit: Some(hit_max),
        material: meshes[mesh_id].material,
        normal,
    };
}
