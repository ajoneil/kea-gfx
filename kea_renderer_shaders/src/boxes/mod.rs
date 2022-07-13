mod boxo;
pub mod entrypoints;

use crate::{ShaderGroupId, SlotId};
pub use boxo::Boxo;
use core::any::TypeId;
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStages, Slot, SlotType},
};

pub const SHADER: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::BoxesHit,
    ShaderGroup::ProceduralHit {
        intersection: Shader("boxes::entrypoints::intersect_box"),
        hit: Shader("boxes::entrypoints::hit_box"),
    },
);

pub const SLOT: (SlotId, Slot) = (
    SlotId::Boxes,
    Slot::new(
        SlotType::Buffer(TypeId::of::<&[Boxo]>()),
        ShaderStages {
            raygen: false,
            intersection: true,
            closest_hit: true,
        },
    ),
);
