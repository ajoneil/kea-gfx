use crate::gpu::{
    buffer::{AllocatedBuffer, Buffer},
    command::CommandBufferRecorder,
    device::Device,
    rasterization_pipeline::RasterizationPipeline,
    swapchain::SwapchainImageView,
};
use ash::vk;
use glam::{vec2, vec3};
use gpu_allocator::MemoryLocation;
use std::{mem, sync::Arc};

pub struct Rasterizer {
    pipeline: RasterizationPipeline,
    vertex_buffer: AllocatedBuffer,
    vertices: Vec<shaders::Vertex>,
}

impl Rasterizer {
    pub fn new(device: Arc<Device>, format: vk::Format) -> Rasterizer {
        let pipeline = RasterizationPipeline::new(device.clone(), format);

        let vertices = vec![
            shaders::Vertex {
                position: vec2(0.0, -0.5),
                color: vec3(1.0, 0.0, 0.0),
            },
            shaders::Vertex {
                position: vec2(0.5, 0.5),
                color: vec3(0.0, 1.0, 0.0),
            },
            shaders::Vertex {
                position: vec2(-0.5, 0.5),
                color: vec3(0.0, 0.0, 1.0),
            },
        ];
        let vertex_buffer = Self::create_vertex_buffer(device, &vertices);

        Rasterizer {
            pipeline,
            vertices,
            vertex_buffer,
        }
    }

    fn create_vertex_buffer(device: Arc<Device>, vertices: &[shaders::Vertex]) -> AllocatedBuffer {
        let buffer = Buffer::new(
            device,
            (mem::size_of::<shaders::Vertex>() * vertices.len()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        let buffer = buffer.allocate("vertices", MemoryLocation::CpuToGpu);
        buffer.fill(vertices);

        buffer
    }

    pub fn draw(&self, cmd: &CommandBufferRecorder, image_view: &SwapchainImageView) {
        cmd.with_render_image_barrier(image_view.image, || {
            cmd.render(&image_view.view, || {
                cmd.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, &self.pipeline.pipeline);
                cmd.bind_vertex_buffers(&[&self.vertex_buffer], 0);
                cmd.draw(self.vertices.len() as u32, 1, 0, 0);
            });
        });
    }
}
