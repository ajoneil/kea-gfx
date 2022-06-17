use std::{ffi::CStr, mem, sync::Arc};

use ash::vk;
use memoffset::offset_of;

use super::{shaders::ShaderModule, Device};

pub struct RasterizationPipeline {
    pub pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,

    device: Arc<Device>,
}

impl RasterizationPipeline {
    pub fn new(device: &Arc<Device>, format: vk::Format) -> RasterizationPipeline {
        let shader_module = ShaderModule::new(device);
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shader_module.module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_vertex\0") })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(shader_module.module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_fragment\0") })
                .build(),
        ];

        let bindings = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(mem::size_of::<shaders::Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()];

        let attributes = [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(shaders::Vertex, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(shaders::Vertex, color) as u32)
                .build(),
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attributes)
            .build();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: 1920,
                height: 1080,
            },
        }];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let layout = unsafe {
            device
                .vk()
                .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::builder(), None)
        }
        .unwrap();

        let formats = [format];
        let mut rendering_info =
            vk::PipelineRenderingCreateInfo::builder().color_attachment_formats(&formats);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .push_next(&mut rendering_info)
            .layout(layout);

        let pipelines = unsafe {
            device.vk().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info.build()],
                None,
            )
        }
        .unwrap();

        RasterizationPipeline {
            pipeline: pipelines[0],
            layout,
            device: device.clone(),
        }
    }
}

impl Drop for RasterizationPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_pipeline(self.pipeline, None);
            self.device.vk().destroy_pipeline_layout(self.layout, None);
        }
    }
}
