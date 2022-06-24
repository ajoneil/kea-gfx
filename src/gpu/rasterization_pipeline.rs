use super::{
    device::Device,
    pipeline::{
        GraphicsPipelineDescription, Pipeline, PipelineDescription, PipelineLayout,
        PipelineShaderStage, PipelineViewport, PipelineViewportState,
    },
    shaders::ShaderModule,
};
use ash::vk;
use memoffset::offset_of;
use std::{mem, sync::Arc};

pub struct RasterizationPipeline {
    pub pipeline: Pipeline,
}

impl RasterizationPipeline {
    pub fn new(device: Arc<Device>, format: vk::Format) -> RasterizationPipeline {
        let shader_module = ShaderModule::new(device.clone());

        let shader_stages = [
            PipelineShaderStage::new(
                vk::ShaderStageFlags::VERTEX,
                &shader_module.entry_point("main_vertex"),
            ),
            PipelineShaderStage::new(
                vk::ShaderStageFlags::FRAGMENT,
                &shader_module.entry_point("main_fragment"),
            ),
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

        let viewport_state = PipelineViewportState::new(&[PipelineViewport {
            viewport: vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
                min_depth: 0.0,
                max_depth: 1.0,
            },
            scissor: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1920,
                    height: 1080,
                },
            },
        }]);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();

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

        let color_attachment_formats = [format];

        let layout = PipelineLayout::new(device.clone(), &[]);
        let mut pipeline_description =
            PipelineDescription::Graphics(GraphicsPipelineDescription::new(
                &shader_stages,
                &vertex_input_state,
                &input_assembly_state,
                &viewport_state,
                &rasterization_state,
                &multisample_state,
                &color_blend_state,
                &color_attachment_formats,
                &layout,
            ));

        let pipeline = Pipeline::new(device, &mut pipeline_description);

        RasterizationPipeline { pipeline }
    }
}
