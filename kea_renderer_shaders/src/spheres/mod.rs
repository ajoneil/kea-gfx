pub mod entrypoints;
mod sphere;

use crate::{ShaderGroupId, SlotId};
use core::any::TypeId;
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStage, Slot, SlotType},
};
pub use sphere::Sphere;

pub const SHADER: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::SphereHit,
    ShaderGroup::ProceduralHit {
        intersection: Shader("spheres::entrypoints::intersect_sphere"),
        hit: Shader("spheres::entrypoints::sphere_hit"),
    },
);

pub const SLOT: (SlotId, Slot) = (
    SlotId::Spheres,
    Slot::new(
        SlotType::Buffer(TypeId::of::<&[Sphere]>()),
        ShaderStage::Intersection,
    ),
);
