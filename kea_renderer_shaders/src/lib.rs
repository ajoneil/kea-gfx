#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// #![deny(warnings)]
#![feature(const_type_id)]
#![feature(asm_experimental_arch)]

use kea_gpu_shaderlib::{shaders::ShaderGroup, slots::Slot};

pub mod boxes;
pub mod cameras;
pub mod lights;
pub mod materials;
pub mod path_tracer;
mod payload;
pub mod spheres;
pub mod triangles;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum SlotId {
    Scene = 0,
    OutputImage,
    Spheres,
    Boxes,
}

impl Into<u32> for SlotId {
    fn into(self) -> u32 {
        self as u32
    }
}

pub const SLOTS: [(SlotId, Slot); 4] = [
    path_tracer::SLOT_SCENE,
    path_tracer::SLOT_OUTPUT_IMAGE,
    spheres::SLOT,
    boxes::SLOT,
];

#[derive(Clone)]
pub enum ShaderGroupId {
    RayGen,
    Miss,
    TriangleHit,
    SphereHit,
    BoxesHit,
}

pub const SHADERS: [(ShaderGroupId, ShaderGroup); 5] = [
    path_tracer::SHADER_GENERATE_RAY,
    path_tracer::SHADER_RAY_MISS,
    triangles::SHADER,
    spheres::SHADER,
    boxes::SHADER,
];
