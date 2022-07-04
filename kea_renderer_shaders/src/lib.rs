#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// #![deny(warnings)]
#![feature(const_type_id)]

use core::any::TypeId;
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStage, Slot, SlotType},
};

// Needed for .sqrt()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub mod sphere;
pub use sphere::Sphere;

#[derive(Clone)]
pub enum SlotId {
    Scene,
    OutputImage,
    Spheres,
}

pub const SLOTS: [(SlotId, Slot); 3] = [
    (
        SlotId::Scene,
        Slot::new(SlotType::AccelerationStructure, ShaderStage::RayGen),
    ),
    (
        SlotId::OutputImage,
        Slot::new(SlotType::Image, ShaderStage::RayGen),
    ),
    (
        SlotId::Spheres,
        Slot::new(
            SlotType::Buffer(TypeId::of::<&[Sphere]>()),
            ShaderStage::Intersection,
        ),
    ),
];

#[derive(Clone)]
pub enum ShaderGroupId {
    RayGen,
    Miss,
    SphereHit,
}

pub const SHADERS: [(ShaderGroupId, ShaderGroup); 3] = [
    (
        ShaderGroupId::RayGen,
        ShaderGroup::RayGeneration(Shader("entrypoints::generate_rays")),
    ),
    (
        ShaderGroupId::Miss,
        ShaderGroup::Miss(Shader("entrypoints::ray_miss")),
    ),
    (
        ShaderGroupId::SphereHit,
        ShaderGroup::ProceduralHit {
            intersection: Shader("entrypoints::intersect_sphere"),
            hit: Shader("entrypoints::ray_hit"),
        },
    ),
];

mod entrypoints;
pub use entrypoints::*;
