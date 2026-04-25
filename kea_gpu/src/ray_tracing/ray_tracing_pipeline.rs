use super::RayTracingShaderBindingTables;
use crate::{
    device::Device,
    pipelines::{Pipeline, PipelineLayout},
    shaders::{PipelineShaders, ShaderGroups},
    slots::SlotLayout,
};
use ash::vk;
use std::{path::PathBuf, slice, sync::Arc};

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
        let cache_path = pipeline_cache_path();
        let cache_data = cache_path
            .as_ref()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default();
        let pipeline_cache = unsafe {
            let create_info =
                vk::PipelineCacheCreateInfo::default().initial_data(&cache_data);
            device
                .raw()
                .create_pipeline_cache(&create_info, None)
                .unwrap()
        };

        let pipeline = unsafe {
            let stages: Vec<vk::PipelineShaderStageCreateInfo> = shaders
                .stages
                .iter()
                .map(|(name, stage, entry_point)| {
                    vk::PipelineShaderStageCreateInfo::default()
                        .stage(*stage)
                        .module(entry_point.module().raw())
                        .name(&name)
                })
                .collect();

            log::debug!("{:?}", stages);
            log::debug!("{:?}", shaders.groups);

            let create_info = vk::RayTracingPipelineCreateInfoKHR::default()
                .stages(&stages)
                .groups(&shaders.groups)
                .max_pipeline_ray_recursion_depth(1)
                .layout(layout.raw());

            let raw = device
                .ext()
                .ray_tracing_pipeline
                .as_ref()
                .unwrap()
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    pipeline_cache,
                    slice::from_ref(&create_info),
                    None,
                )
                .unwrap()
                .into_iter()
                .nth(0)
                .unwrap();

            Pipeline::new(device.clone(), raw)
        };

        if let Some(path) = cache_path.as_ref() {
            unsafe {
                if let Ok(data) = device.raw().get_pipeline_cache_data(pipeline_cache) {
                    if data != cache_data {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Err(e) = std::fs::write(path, &data) {
                            log::warn!("failed to write pipeline cache to {:?}: {}", path, e);
                        } else {
                            log::debug!(
                                "wrote pipeline cache ({} bytes) to {:?}",
                                data.len(),
                                path
                            );
                        }
                    }
                }
            }
        }
        unsafe { device.raw().destroy_pipeline_cache(pipeline_cache, None) };

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

fn pipeline_cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("kea-gfx").join("pipeline-cache.bin"))
}
