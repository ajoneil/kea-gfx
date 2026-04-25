use crate::{ShaderGroupId, SlotId};
use core::any::TypeId;
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStages, Slot, SlotType},
};

pub mod entrypoints;
mod mesh;

pub use mesh::Mesh;

pub const SHADER: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::TriangleHit,
    ShaderGroup::TriangleHit(Shader("triangles::entrypoints::triangle_hit")),
);

pub const SLOT_MESHES: (SlotId, Slot) = (
    SlotId::Meshes,
    Slot::new(
        SlotType::Buffer(TypeId::of::<&[Mesh]>()),
        ShaderStages {
            raygen: false,
            intersection: false,
            closest_hit: true,
        },
    ),
);
