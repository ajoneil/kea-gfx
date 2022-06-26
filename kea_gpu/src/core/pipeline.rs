use super::{descriptor_set::DescriptorSetLayout, device::Device, shaders::ShaderEntryPoint};
use ash::vk;
use std::{ffi::CString, marker::PhantomData, pin::Pin, slice, sync::Arc};

pub struct PipelineLayout {
    device: Arc<Device>,
    raw: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: Arc<Device>, set_layouts: &[DescriptorSetLayout]) -> PipelineLayout {
        let layouts: Vec<vk::DescriptorSetLayout> =
            set_layouts.iter().map(|dsl| unsafe { dsl.raw() }).collect();
        let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&layouts);

        let raw = unsafe { device.raw().create_pipeline_layout(&create_info, None) }.unwrap();

        PipelineLayout { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.raw, None);
        }
    }
}

pub struct PipelineShaderStage<'a> {
    stage: vk::ShaderStageFlags,
    entry_point: &'a ShaderEntryPoint<'a>,
    entry_point_name: Pin<CString>,
}

impl<'a> PipelineShaderStage<'a> {
    pub fn new(
        stage: vk::ShaderStageFlags,
        entry_point: &'a ShaderEntryPoint<'a>,
    ) -> PipelineShaderStage<'a> {
        let entry_point_name = Pin::new(CString::new(entry_point.name().clone()).unwrap());

        PipelineShaderStage {
            stage,
            entry_point,
            entry_point_name,
        }
    }

    pub unsafe fn raw(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(self.stage)
            .module(self.entry_point.module().raw())
            .name(&self.entry_point_name)
            .build()
    }
}

pub struct Pipeline {
    device: Arc<Device>,
    raw: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: Arc<Device>, pipeline_description: &PipelineDescription) -> Pipeline {
        let raw = match pipeline_description {
            PipelineDescription::RayTracing(desc) => unsafe {
                device
                    .ext()
                    .ray_tracing_pipeline
                    .create_ray_tracing_pipelines(
                        vk::DeferredOperationKHR::null(),
                        vk::PipelineCache::null(),
                        slice::from_ref(&desc.raw()),
                        None,
                    )
            }
            .unwrap()[0],
        };

        Pipeline { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::Pipeline {
        self.raw
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline(self.raw, None);
        }
    }
}

pub enum PipelineDescription<'a> {
    RayTracing(RayTracingPipelineDescription<'a>),
}

pub struct RayTracingPipelineDescription<'a> {
    raw: vk::RayTracingPipelineCreateInfoKHR,
    _raw_stages: Vec<vk::PipelineShaderStageCreateInfo>,
    marker: PhantomData<&'a ()>,
}

impl<'a> RayTracingPipelineDescription<'a> {
    pub fn new(
        stages: &[PipelineShaderStage<'a>],
        groups: &'a [vk::RayTracingShaderGroupCreateInfoKHR],
        layout: &'a PipelineLayout,
    ) -> RayTracingPipelineDescription<'a> {
        let raw_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            stages.iter().map(|ss| unsafe { ss.raw() }).collect();

        let raw = vk::RayTracingPipelineCreateInfoKHR::builder()
            .stages(&raw_stages)
            .groups(groups)
            .max_pipeline_ray_recursion_depth(1)
            .layout(unsafe { layout.raw() })
            .build();

        RayTracingPipelineDescription {
            raw,
            _raw_stages: raw_stages,
            marker: PhantomData,
        }
    }

    pub unsafe fn raw(&self) -> vk::RayTracingPipelineCreateInfoKHR {
        self.raw
    }
}
