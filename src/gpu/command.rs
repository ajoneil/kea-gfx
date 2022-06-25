use super::{
    buffer::AllocatedBuffer,
    descriptor_set::DescriptorSet,
    device::{Device, Queue},
    pipeline::{Pipeline, PipelineLayout},
    rt::{
        acceleration_structure::{AccelerationStructure, AccelerationStructureDescription},
        shader_binding_table::RayTracingShaderBindingTables,
    },
    sync::Fence,
};
use ash::vk;
use log::info;
use std::sync::Arc;

pub struct CommandPool {
    pool: vk::CommandPool,
    queue: Queue,
}

impl CommandPool {
    pub fn new(queue: Queue) -> CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.family().index());
        let pool = unsafe { queue.device().vk().create_command_pool(&create_info, None) }.unwrap();

        CommandPool { pool, queue }
    }

    pub fn allocate_buffer(self: &Arc<Self>) -> CommandBuffer {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let buffer =
            unsafe { self.device().vk().allocate_command_buffers(&create_info) }.unwrap()[0];

        CommandBuffer {
            buffer,
            pool: self.clone(),
        }
    }

    fn device(&self) -> &Arc<Device> {
        self.queue.device()
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device().queue_wait_idle(&self.queue);
            self.device().vk().destroy_command_pool(self.pool, None);
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
            self.device()
                .vk()
                .reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::empty())
        }
        .unwrap();
    }

    fn begin(&self) {
        unsafe {
            self.device()
                .vk()
                .begin_command_buffer(self.buffer, &vk::CommandBufferBeginInfo::default())
        }
        .unwrap();
    }

    fn end(&self) {
        unsafe { self.device().vk().end_command_buffer(self.buffer) }.unwrap()
    }

    pub fn device(&self) -> &Arc<Device> {
        self.pool.device()
    }

    pub fn submit(&self) -> Fence {
        self.pool.queue.submit(&[self])
    }
}

pub struct CommandBufferRecorder<'a> {
    buffer: &'a CommandBuffer,
}

impl CommandBufferRecorder<'_> {
    pub fn device(&self) -> &Arc<Device> {
        self.buffer.device()
    }

    pub fn bind_pipeline(&self, bind_point: vk::PipelineBindPoint, pipeline: &Pipeline) {
        unsafe {
            self.device()
                .vk()
                .cmd_bind_pipeline(self.buffer.buffer, bind_point, pipeline.raw())
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        bind_point: vk::PipelineBindPoint,
        layout: &PipelineLayout,
        descriptor_sets: &[DescriptorSet],
    ) {
        let raw_sets: Vec<vk::DescriptorSet> = descriptor_sets
            .into_iter()
            .map(|ds| unsafe { ds.raw() })
            .collect();

        unsafe {
            self.device().vk().cmd_bind_descriptor_sets(
                self.buffer.buffer,
                bind_point,
                layout.raw(),
                0,
                &raw_sets,
                &[],
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
            self.device().vk().cmd_pipeline_barrier(
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

    pub fn transition_image_layout(
        &self,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
    ) {
        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
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
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    pub fn copy_image(&self, from: vk::Image, to: vk::Image, region: &vk::ImageCopy) {
        unsafe {
            self.device().vk().cmd_copy_image(
                self.buffer.buffer,
                from,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                to,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(region),
            )
        };
    }

    pub fn build_acceleration_structure(
        &self,
        description: &AccelerationStructureDescription,
        destination: &AccelerationStructure,
        scratch: &AllocatedBuffer,
    ) {
        let description = description.bind_for_build(destination, scratch);
        info!("geo: {:?}", unsafe {
            *description.geometry_info().p_geometries
        });
        info!("ranges: {:?}", description.ranges());
        unsafe {
            self.device()
                .ext
                .acceleration_structure
                .cmd_build_acceleration_structures(
                    self.buffer.buffer,
                    &[description.geometry_info()],
                    &[description.ranges()],
                );
        }
    }

    pub fn trace_rays(
        &self,
        binding_tables: &RayTracingShaderBindingTables,
        width: u32,
        height: u32,
        depth: u32,
    ) {
        // info!("binding tables: {:?}", binding_tables);
        unsafe {
            self.device().ext.ray_tracing_pipeline.cmd_trace_rays(
                self.buffer.buffer,
                binding_tables.raygen.raw(),
                binding_tables.miss.raw(),
                binding_tables.hit.raw(),
                binding_tables.callable.raw(),
                width,
                height,
                depth,
            )
        }
    }
}
