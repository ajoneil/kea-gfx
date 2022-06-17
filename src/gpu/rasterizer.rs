use std::{mem, sync::Arc};

use ash::vk;
use glam::{vec2, vec3};
use gpu_allocator::MemoryLocation;

use super::{
    buffer::{AllocatedBuffer, Buffer},
    command::{CommandBuffer, CommandBufferRecorder, CommandPool},
    rasterization_pipeline::RasterizationPipeline,
    swapchain::SwapchainImageView,
    sync::{Fence, Semaphore},
    Device, Swapchain,
};

struct Semaphores {
    image_available: Semaphore,
    render_finished: Semaphore,
}

pub struct Rasterizer {
    swapchain: Swapchain,
    pipeline: RasterizationPipeline,
    command_buffer: CommandBuffer,
    semaphores: Semaphores,
    in_flight_fence: Fence,
    vertex_buffer: AllocatedBuffer,

    vertices: Vec<shaders::Vertex>,
}

impl Rasterizer {
    pub fn new(swapchain: Swapchain) -> Rasterizer {
        let pipeline = RasterizationPipeline::new(&swapchain.device, swapchain.format);
        let semaphores = Semaphores {
            image_available: Semaphore::new(swapchain.device.clone()),
            render_finished: Semaphore::new(swapchain.device.clone()),
        };
        let in_flight_fence = Fence::new(swapchain.device.clone(), true);

        let command_buffer = Arc::new(CommandPool::new(swapchain.device.clone())).allocate_buffer();
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
        let vertex_buffer = Self::create_vertex_buffer(&swapchain.device, &vertices);

        Rasterizer {
            swapchain,
            pipeline,
            command_buffer,
            semaphores,
            in_flight_fence,

            vertices,
            vertex_buffer,
        }
    }

    fn record_command_buffer(&self, image_view: &SwapchainImageView) {
        self.command_buffer.record(true, |cmd| {
            self.with_render_image_barrier(cmd, image_view.image, || {
                cmd.render(&image_view.view, || {
                    cmd.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);
                    cmd.bind_vertex_buffers(&[&self.vertex_buffer], 0);
                    cmd.draw(self.vertices.len() as u32, 1, 0, 0);
                });
            });
        });
    }

    pub fn draw(&self) {
        self.in_flight_fence.wait_and_reset();

        let (image_index, image_view) = self
            .swapchain
            .acquire_next_image(&self.semaphores.image_available);

        self.record_command_buffer(&image_view);

        unsafe {
            let submits = [vk::SubmitInfo::builder()
                .wait_semaphores(&[self.semaphores.image_available.vk()])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.command_buffer.buffer])
                .signal_semaphores(&[self.semaphores.render_finished.vk()])
                .build()];

            self.swapchain
                .device
                .vk()
                .queue_submit(
                    self.swapchain.device.queue,
                    &submits,
                    self.in_flight_fence.vk(),
                )
                .unwrap();

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[self.semaphores.render_finished.vk()])
                .swapchains(&[self.swapchain.swapchain])
                .image_indices(&[image_index])
                .build();

            self.swapchain
                .device
                .ext
                .swapchain
                .queue_present(self.swapchain.device.queue, &present)
                .unwrap();
        }
    }

    fn with_render_image_barrier<F>(&self, cmd: &CommandBufferRecorder, image: vk::Image, func: F)
    where
        F: FnOnce(),
    {
        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image(image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .build();

        cmd.pipeline_barrier(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        func();

        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .image(image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .build();

        cmd.pipeline_barrier(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    fn create_vertex_buffer(device: &Arc<Device>, vertices: &[shaders::Vertex]) -> AllocatedBuffer {
        let buffer = Buffer::new(
            device,
            (mem::size_of::<shaders::Vertex>() * vertices.len()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        let buffer = buffer.allocate("vertices", MemoryLocation::CpuToGpu, true);
        buffer.fill(vertices);

        buffer
    }
}

impl Drop for Rasterizer {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.device.vk().device_wait_idle().unwrap();
        }
    }
}
