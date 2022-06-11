#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

#[spirv(vertex)]
pub fn main_vertex(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    fragment_color: &mut Vec3,
) {
    let positions: [Vec2; 3] = [vec2(0.0, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5)];
    let colors: [Vec3; 3] = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ];

    *out_pos = vec4(
        positions[vert_id as usize].x,
        positions[vert_id as usize].y,
        0.0,
        1.0,
    );

    *fragment_color = colors[vert_id as usize];
}

#[spirv(fragment)]
pub fn main_fragment(fragment_color: Vec3, output: &mut Vec4) {
    *output = vec4(fragment_color.x, fragment_color.y, fragment_color.z, 1.0);
}
