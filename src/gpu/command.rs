use super::{
    buffer::AllocatedBuffer, device::{Device, Queue}, rt::acceleration_structure::Blas, swapchain::ImageView,
};
use ash::vk;
use std::sync::Arc;

pub struct CommandPool {
    pool: vk::CommandPool,
    device: Arc<Device>,
    queue: Queue
}

impl CommandPool {
    pub fn new(device: Arc<Device>, queue: Queue) -> CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.family_index());
        let pool = unsafe { device.vk().create_command_pool(&create_info, None) }.unwrap();

        CommandPool { pool, device, queue }
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
            self.device.queue_wait_idle(&self.queue);
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
    pub fn render<F>(&self, image_view: &ImageView, func: F)
    where
        F: FnOnce(),
    {
        self.begin_rendering(image_view);

        func();

        self.end_rendering();
    }

    fn begin_rendering(&self, image_view: &ImageView) {
        let color_attachments = [vk::RenderingAttachmentInfo::builder()
            .image_view(unsafe { image_view.vk() })
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            })
            .store_op(vk::AttachmentStoreOp::STORE)
            .build()];
        let rendering_info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1920,
                    height: 1080,
                },
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        unsafe {
            self.buffer
                .pool
                .device
                .vk()
                .cmd_begin_rendering(self.buffer.buffer, &rendering_info);
        }
    }

    fn end_rendering(&self) {
        unsafe {
            self.buffer
                .pool
                .device
                .vk()
                .cmd_end_rendering(self.buffer.buffer)
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

    pub fn bind_vertex_buffers(&self, buffers: &[&AllocatedBuffer], first_binding: u32) {
        let buffers: Vec<vk::Buffer> = buffers.iter().map(|b| unsafe { b.buffer().vk() }).collect();
        let offsets: Vec<vk::DeviceSize> = buffers.iter().map(|_| 0).collect();
        unsafe {
            self.buffer.pool.device.vk().cmd_bind_vertex_buffers(
                self.buffer.buffer,
                first_binding,
                &buffers,
                &offsets,
            );
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

    pub fn pipeline_barrier(
        &self,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.buffer.pool.device.vk().cmd_pipeline_barrier(
                self.buffer.buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn with_render_image_barrier<F>(&self, image: vk::Image, func: F)
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

        self.pipeline_barrier(
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

        self.pipeline_barrier(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    pub fn build_blas(
        &self,
        blas: &Blas,
        src: vk::AccelerationStructureKHR,
        dst: vk::AccelerationStructureKHR,
        scratch: &AllocatedBuffer,
    ) {
        let blas = blas.bind_for_build(src, dst, scratch);
        unsafe {
            self.buffer
                .pool
                .device
                .ext
                .acceleration_structure
                .cmd_build_acceleration_structures(
                    self.buffer.buffer,
                    &[blas.geometry_info()],
                    &[blas.ranges()],
                );
        }
    }
}
