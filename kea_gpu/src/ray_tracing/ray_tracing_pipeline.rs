use super::RayTracingShaderBindingTables;
use crate::{
    device::Device,
    pipelines::{Pipeline, PipelineLayout},
    shaders::{PipelineShaders, ShaderGroups},
    slots::SlotLayout,
};
use ash::vk;
use std::{slice, sync::Arc};

pub struct RayTracingPipeline<SlotId> {
    _shaders: PipelineShaders,
    layout: PipelineLayout,
    slot_layout: SlotLayout<SlotId>,
    pipeline: Pipeline,
    shader_binding_tables: RayTracingShaderBindingTables,
}

impl<SlotId> RayTracingPipeline<SlotId> {
    pub fn new<ShaderGroupId>(
        device: Arc<Device>,
        shader_groups: ShaderGroups<ShaderGroupId>,
        shaders: PipelineShaders,
        layout: PipelineLayout,
        slot_layout: SlotLayout<SlotId>,
    ) -> Self {
        let pipeline = unsafe {
            let stages: Vec<vk::PipelineShaderStageCreateInfo> = shaders
                .stages
                .iter()
                .map(|(name, stage, entry_point)| {
                    vk::PipelineShaderStageCreateInfo::builder()
                        .stage(*stage)
                        .module(entry_point.module().raw())
                        .name(&name)
                        .build()
                })
                .collect();

            let create_info = vk::RayTracingPipelineCreateInfoKHR::builder()
                .stages(&stages)
                .groups(&shaders.groups)
                .max_pipeline_ray_recursion_depth(1)
                .layout(layout.raw())
                .build();

            let raw = device
                .ext()
                .ray_tracing_pipeline
                .as_ref()
                .unwrap()
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    slice::from_ref(&create_info),
                    None,
                )
                .unwrap()
                .into_iter()
                .nth(0)
                .unwrap();

            Pipeline::new(device.clone(), raw)
        };

        let shader_binding_tables =
            RayTracingShaderBindingTables::new(&device, &shader_groups, &shaders, &pipeline);

        Self {
            _shaders: shaders,
            layout,
            slot_layout,
            pipeline,
            shader_binding_tables,
        }
    }

    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn layout(&self) -> &PipelineLayout {
        &self.layout
    }

    pub fn slot_layout(&self) -> &SlotLayout<SlotId> {
        &self.slot_layout
    }

    pub fn shader_binding_tables(&self) -> &RayTracingShaderBindingTables {
        &self.shader_binding_tables
    }
}
