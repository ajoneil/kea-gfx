use kea_gpu_shaderlib::shaders::{Shader, ShaderGroup};

use crate::ShaderGroupId;

pub mod entrypoints;

pub const SHADER: (ShaderGroupId, ShaderGroup) = (
    ShaderGroupId::TriangleHit,
    ShaderGroup::TriangleHit(Shader("triangles::entrypoints::triangle_hit")),
);
