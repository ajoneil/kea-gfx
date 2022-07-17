use crate::{ShaderGroupId, SlotId};
use core::any::TypeId;
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStages, Slot, SlotType},
};

pub mod entrypoints;
mod mesh;

pub use mesh::Mesh;
use spirv_std::glam::{UVec3, Vec3A};

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

pub const SLOT_VERTICES: (SlotId, Slot) = (
    SlotId::Vertices,
    Slot::new(
        SlotType::Buffer(TypeId::of::<&[Vec3A]>()),
        ShaderStages {
            raygen: false,
            intersection: false,
            closest_hit: true,
        },
    ),
);

pub const SLOT_INDICES: (SlotId, Slot) = (
    SlotId::Indices,
    Slot::new(
        SlotType::Buffer(TypeId::of::<&[UVec3]>()),
        ShaderStages {
            raygen: false,
            intersection: false,
            closest_hit: true,
        },
    ),
);
