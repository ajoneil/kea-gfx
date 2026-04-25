use spirv_std::spirv;

use super::Mesh;
use crate::payload::RayPayload;
use spirv_std::glam::{vec3, Vec3, Vec3A};

#[spirv(closest_hit)]
pub fn triangle_hit(
    #[spirv(ray_tmax)] hit_max: f32,
    #[spirv(world_ray_direction)] ray_direction: Vec3,
    #[spirv(incoming_ray_payload)] ray_payload: &mut RayPayload,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] meshes: &[Mesh],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] vertices: &[Vec3A],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] indices: &[[u32; 3]],
    #[spirv(instance_custom_index)] mesh_id: usize,
    #[spirv(ray_geometry_index)] index: u32,
) {
    let mesh = meshes[mesh_id];
    let index = indices[(mesh.indices_offset + index) as usize];
    let points = [
        vertices[(mesh.vertices_offset + index[0]) as usize],
        vertices[(mesh.vertices_offset + index[1]) as usize],
        vertices[(mesh.vertices_offset + index[2]) as usize],
    ];

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
        material: mesh.material,
        normal,
    };
}
