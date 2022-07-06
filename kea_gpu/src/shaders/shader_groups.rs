use super::{ShaderEntryPoint, ShaderModule};
use crate::device::Device;
use ash::vk;
use kea_gpu_shaderlib::shaders::{Shader, ShaderGroup};
use std::{collections::HashMap, ffi::CString, sync::Arc};

pub struct ShaderGroups<ShaderGroupId> {
    groups: Vec<(ShaderGroupId, ShaderGroup)>,
}

impl<ShaderGroupId> ShaderGroups<ShaderGroupId> {
    pub fn new(groups: Vec<(ShaderGroupId, ShaderGroup)>) -> Self {
        Self { groups }
    }

    pub fn build(&self, device: Arc<Device>, shader_crate_path: &str) -> PipelineShaders {
        let modules = ShaderModule::new_multimodule(&device, shader_crate_path);
        let mut stages: Vec<(CString, vk::ShaderStageFlags, ShaderEntryPoint)> = vec![];
        let groups: Vec<vk::RayTracingShaderGroupCreateInfoKHR> = self
            .groups
            .iter()
            .map(|(_, group)| match group {
                ShaderGroup::RayGeneration(Shader(shader)) => {
                    let entry_point = modules[*shader].entry_point(shader);
                    let index = stages.len();
                    let stage = (
                        CString::new(*shader).unwrap(),
                        vk::ShaderStageFlags::RAYGEN_KHR,
                        entry_point,
                    );
                    stages.push(stage);
                    vk::RayTracingShaderGroupCreateInfoKHR::builder()
                        .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                        .general_shader(index as _)
                        .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                        .any_hit_shader(vk::SHADER_UNUSED_KHR)
                        .intersection_shader(vk::SHADER_UNUSED_KHR)
                        .build()
                }
                ShaderGroup::Miss(Shader(shader)) => {
                    let entry_point = modules[*shader].entry_point(shader);
                    let index = stages.len();
                    let stage = (
                        CString::new(*shader).unwrap(),
                        vk::ShaderStageFlags::MISS_KHR,
                        entry_point,
                    );
                    stages.push(stage);
                    vk::RayTracingShaderGroupCreateInfoKHR::builder()
                        .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                        .general_shader(index as _)
                        .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                        .any_hit_shader(vk::SHADER_UNUSED_KHR)
                        .intersection_shader(vk::SHADER_UNUSED_KHR)
                        .build()
                }
                ShaderGroup::ProceduralHit {
                    intersection: Shader(intersection),
                    hit: Shader(hit),
                } => {
                    let intersection_entry_point = modules[*intersection].entry_point(intersection);
                    let intersection_index = stages.len();
                    let stage = (
                        CString::new(*intersection).unwrap(),
                        vk::ShaderStageFlags::INTERSECTION_KHR,
                        intersection_entry_point,
                    );
                    stages.push(stage);
                    let hit_entry_point = modules[*hit].entry_point(hit);
                    let hit_index = stages.len();
                    let stage = (
                        CString::new(*hit).unwrap(),
                        vk::ShaderStageFlags::CLOSEST_HIT_KHR,
                        hit_entry_point,
                    );
                    stages.push(stage);
                    vk::RayTracingShaderGroupCreateInfoKHR::builder()
                        .ty(vk::RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP)
                        .general_shader(vk::SHADER_UNUSED_KHR)
                        .closest_hit_shader(hit_index as _)
                        .any_hit_shader(vk::SHADER_UNUSED_KHR)
                        .intersection_shader(intersection_index as _)
                        .build()
                }
                ShaderGroup::TriangleHit(Shader(shader)) => {
                    let entry_point = modules[*shader].entry_point(shader);
                    let index = stages.len();
                    let stage = (
                        CString::new(*shader).unwrap(),
                        vk::ShaderStageFlags::CLOSEST_HIT_KHR,
                        entry_point,
                    );
                    stages.push(stage);
                    vk::RayTracingShaderGroupCreateInfoKHR::builder()
                        .ty(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                        .general_shader(vk::SHADER_UNUSED_KHR)
                        .closest_hit_shader(index as _)
                        .any_hit_shader(vk::SHADER_UNUSED_KHR)
                        .intersection_shader(vk::SHADER_UNUSED_KHR)
                        .build()
                }
            })
            .collect();

        PipelineShaders {
            modules,
            stages,
            groups,
        }
    }

    pub fn groups(&self) -> &[(ShaderGroupId, ShaderGroup)] {
        &self.groups
    }
}

pub struct PipelineShaders {
    pub modules: HashMap<String, Arc<ShaderModule>>,
    pub stages: Vec<(CString, vk::ShaderStageFlags, ShaderEntryPoint)>,
    pub groups: Vec<vk::RayTracingShaderGroupCreateInfoKHR>,
}
