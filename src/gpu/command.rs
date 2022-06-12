use std::sync::Arc;

use ash::vk;

use super::Device;

pub struct CommandPool {
    pool: vk::CommandPool,
    device: Arc<Device>,
}

impl CommandPool {
    pub fn new(device: Arc<Device>) -> CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue_family_index);
        let pool = unsafe { device.vk().create_command_pool(&create_info, None) }.unwrap();

        CommandPool { pool, device }
    }

    pub fn allocate_buffer(self: Arc<Self>) -> CommandBuffer {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let buffer = unsafe { self.device.vk().allocate_command_buffers(&create_info) }.unwrap()[0];

        CommandBuffer { buffer, pool: self }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.pool, None);
        }
    }
}

pub struct CommandBuffer {
    pub buffer: vk::CommandBuffer,
    pool: Arc<CommandPool>,
}

impl CommandBuffer {
    pub fn record<F>(&self, reset: bool, func: F)
    where
        F: FnOnce(&CommandBufferRecorder),
    {
        if reset {
            self.reset()
        }

        self.begin();
        func(&CommandBufferRecorder { buffer: self });
        self.end();
    }

    pub fn reset(&self) {
        unsafe {
            self.pool
                .device
                .vk()
                .reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::empty())
        }
        .unwrap();
    }

    fn begin(&self) {
        unsafe {
            self.pool
                .device
                .vk()
                .begin_command_buffer(self.buffer, &vk::CommandBufferBeginInfo::default())
        }
        .unwrap();
    }

    fn end(&self) {
        unsafe { self.pool.device.vk().end_command_buffer(self.buffer) }.unwrap()
    }
}

pub struct CommandBufferRecorder<'a> {
    buffer: &'a CommandBuffer,
}

impl CommandBufferRecorder<'_> {
    pub fn render_pass<F>(&self, render_pass: vk::RenderPass, framebuffer: vk::Framebuffer, func: F)
    where
        F: FnOnce(),
    {
        self.begin_render_pass(render_pass, framebuffer);

        func();

        self.end_render_pass();
    }

    fn begin_render_pass(&self, render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) {
        let begin_render_pass = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1920,
                    height: 1080,
                },
            })
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }]);

        unsafe {
            self.buffer.pool.device.vk().cmd_begin_render_pass(
                self.buffer.buffer,
                &begin_render_pass,
                vk::SubpassContents::INLINE,
            )
        }
    }

    fn end_render_pass(&self) {
        unsafe {
            self.buffer
                .pool
                .device
                .vk()
                .cmd_end_render_pass(self.buffer.buffer)
        }
    }

    pub fn bind_pipeline(&self, bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self.buffer
                .pool
                .device
                .vk()
                .cmd_bind_pipeline(self.buffer.buffer, bind_point, pipeline)
        }
    }

    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.buffer.pool.device.vk().cmd_draw(
                self.buffer.buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }
}
