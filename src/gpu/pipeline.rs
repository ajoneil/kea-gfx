use super::{descriptor_set::DescriptorSetLayout, device::Device, shaders::ShaderEntryPoint};
use ash::vk;
use std::{ffi::CString, marker::PhantomData, ptr, sync::Arc};

pub struct PipelineLayout {
    device: Arc<Device>,
    raw: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: Arc<Device>, set_layouts: &[DescriptorSetLayout]) -> PipelineLayout {
        let layouts: Vec<vk::DescriptorSetLayout> =
            set_layouts.iter().map(|dsl| unsafe { dsl.raw() }).collect();
        let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&layouts);

        let raw = unsafe { device.vk().create_pipeline_layout(&create_info, None) }.unwrap();

        PipelineLayout { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_pipeline_layout(self.raw, None);
        }
    }
}

pub struct PipelineShaderStage<'a> {
    raw: vk::PipelineShaderStageCreateInfo,
    _entry_point_name: CString,
    marker: PhantomData<&'a ()>,
}

impl<'a> PipelineShaderStage<'a> {
    pub fn new(
        stage: vk::ShaderStageFlags,
        entry_point: &ShaderEntryPoint<'a>,
    ) -> PipelineShaderStage<'a> {
        let entry_point_name = CString::new(entry_point.name().clone()).unwrap();
        let raw = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .module(unsafe { entry_point.module().raw() })
            .name(&entry_point_name)
            .build();

        PipelineShaderStage {
            raw,
            _entry_point_name: entry_point_name,
            marker: PhantomData,
        }
    }

    pub unsafe fn raw(&self) -> vk::PipelineShaderStageCreateInfo {
        self.raw
    }
}

pub struct Pipeline {
    device: Arc<Device>,
    raw: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: Arc<Device>, pipeline_description: &PipelineDescription) -> Pipeline {
        let raw = match pipeline_description {
            PipelineDescription::Graphics(desc) => unsafe {
                device.vk().create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[desc.raw()],
                    None,
                )
            }
            .unwrap()[0],
            PipelineDescription::RayTracing(desc) => unsafe {
                device
                    .ext
                    .ray_tracing_pipeline
                    .create_ray_tracing_pipelines(
                        vk::DeferredOperationKHR::null(),
                        vk::PipelineCache::null(),
                        &[desc.raw()],
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
            self.device.vk().destroy_pipeline(self.raw, None);
        }
    }
}

pub enum PipelineDescription<'a> {
    Graphics(GraphicsPipelineDescription<'a>),
    RayTracing(RayTracingPipelineDescription<'a>),
}

pub struct GraphicsPipelineDescription<'a> {
    raw: vk::GraphicsPipelineCreateInfo,
    _raw_stages: Vec<vk::PipelineShaderStageCreateInfo>,
    rendering_info: vk::PipelineRenderingCreateInfo,
    marker: PhantomData<&'a ()>,
}

impl<'a> GraphicsPipelineDescription<'a> {
    pub fn new(
        stages: &[PipelineShaderStage<'a>],
        vertex_input_state: &'a vk::PipelineVertexInputStateCreateInfo,
        input_assembly_state: &'a vk::PipelineInputAssemblyStateCreateInfo,
        viewport_state: &'a PipelineViewportState,
        rasterization_state: &'a vk::PipelineRasterizationStateCreateInfo,
        multisample_state: &'a vk::PipelineMultisampleStateCreateInfo,
        color_blend_state: &'a vk::PipelineColorBlendStateCreateInfo,
        color_attachment_formats: &'a [vk::Format],
        layout: &'a PipelineLayout,
    ) -> GraphicsPipelineDescription<'a> {
        let raw_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            stages.iter().map(|ss| unsafe { ss.raw() }).collect();

        let rendering_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(color_attachment_formats)
            .build();

        let raw = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&raw_stages)
            .vertex_input_state(vertex_input_state)
            .input_assembly_state(input_assembly_state)
            .viewport_state(unsafe { viewport_state.raw() })
            .rasterization_state(rasterization_state)
            .multisample_state(multisample_state)
            .color_blend_state(color_blend_state)
            .layout(unsafe { layout.raw() })
            .build();

        GraphicsPipelineDescription {
            raw,
            _raw_stages: raw_stages,
            rendering_info,
            marker: PhantomData,
        }
    }

    pub unsafe fn raw(&self) -> vk::GraphicsPipelineCreateInfo {
        vk::GraphicsPipelineCreateInfo {
            p_next: ptr::addr_of!(self.rendering_info) as _,
            ..self.raw
        }
    }
}

pub struct PipelineViewport {
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
}

pub struct PipelineViewportState {
    raw: vk::PipelineViewportStateCreateInfo,
    _viewports: Vec<vk::Viewport>,
    _scissors: Vec<vk::Rect2D>,
}

impl PipelineViewportState {
    pub fn new(pipeline_viewports: &[PipelineViewport]) -> PipelineViewportState {
        let viewports: Vec<vk::Viewport> =
            pipeline_viewports.iter().map(|pv| pv.viewport).collect();
        let scissors: Vec<vk::Rect2D> = pipeline_viewports.iter().map(|pv| pv.scissor).collect();

        let raw = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors)
            .build();

        PipelineViewportState {
            raw,
            _viewports: viewports,
            _scissors: scissors,
        }
    }

    pub unsafe fn raw(&self) -> &vk::PipelineViewportStateCreateInfo {
        &self.raw
    }
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
