use crate::{ShaderGroupId, SlotId};
use kea_gpu_shaderlib::{
    shaders::{Shader, ShaderGroup},
    slots::{ShaderStages, Slot, SlotType},
};
pub mod entrypoints;

pub const SHADER_GENERATE_RAY: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::RayGen,
    ShaderGroup::RayGeneration(Shader("path_tracer::entrypoints::generate_rays")),
);

pub const SHADER_RAY_MISS: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::Miss,
    ShaderGroup::Miss(Shader("path_tracer::entrypoints::ray_miss")),
);

pub const SLOT_SCENE: (SlotId, Slot) = (
    SlotId::Scene,
    Slot::new(
        SlotType::AccelerationStructure,
        ShaderStages {
            raygen: true,
            intersection: false,
            closest_hit: false,
        },
    ),
);

pub const SLOT_OUTPUT_IMAGE: (SlotId, Slot) = (
    SlotId::OutputImage,
    Slot::new(
        SlotType::Image,
        ShaderStages {
            raygen: true,
            intersection: false,
            closest_hit: false,
        },
    ),
);
