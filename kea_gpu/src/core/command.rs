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
use log::{info, warn};
use std::{mem::ManuallyDrop, sync::Arc};

pub struct CommandPool {
    queue: Queue,
    raw: vk::CommandPool,
}

impl CommandPool {
    pub fn new(queue: Queue) -> Arc<CommandPool> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.family().index());
        let raw = unsafe { queue.device().raw().create_command_pool(&create_info, None) }.unwrap();

        Arc::new(CommandPool { queue, raw })
    }

    pub fn allocate_buffers(self: &Arc<Self>, count: u32) -> Vec<CommandBuffer> {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.raw)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let raws = unsafe { self.device().raw().allocate_command_buffers(&create_info) }.unwrap();

        raws.into_iter()
            .map(|raw| CommandBuffer {
                pool: self.clone(),
                raw,
            })
            .collect()
    }

    pub fn allocate_buffer(self: &Arc<Self>) -> CommandBuffer {
        self.allocate_buffers(1).into_iter().nth(0).unwrap()
    }

    fn device(&self) -> &Arc<Device> {
        self.queue.device()
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device()
                .raw()
                .queue_wait_idle(self.queue.raw())
                .unwrap();
            self.device().raw().destroy_command_pool(self.raw, None);
        }
    }
}

pub struct CommandBuffer {
    raw: vk::CommandBuffer,
    pool: Arc<CommandPool>,
}

impl CommandBuffer {
    pub fn record<F>(self, func: F) -> RecordedCommandBuffer
    where
        F: FnOnce(&CommandBufferRecorder),
    {
        self.begin();
        func(&CommandBufferRecorder { buffer: &self });
        self.end();

        RecordedCommandBuffer {
            buffer: ManuallyDrop::new(Some(self)),
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device()
                .raw()
                .reset_command_buffer(self.raw, vk::CommandBufferResetFlags::empty())
        }
        .unwrap();
    }

    fn begin(&self) {
        unsafe {
            self.device()
                .raw()
                .begin_command_buffer(self.raw, &vk::CommandBufferBeginInfo::default())
        }
        .unwrap();
    }

    fn end(&self) {
        unsafe { self.device().raw().end_command_buffer(self.raw) }.unwrap()
    }

    pub fn device(&self) -> &Arc<Device> {
        self.pool.device()
    }

    pub unsafe fn raw(&self) -> vk::CommandBuffer {
        self.raw
    }
}

#[must_use]
pub struct RecordedCommandBuffer {
    buffer: ManuallyDrop<Option<CommandBuffer>>,
}

impl RecordedCommandBuffer {
    pub fn submit(self) -> SubmittedCommandBuffer {
        let buffer = unsafe { self.consume() };
        let fence = buffer.pool.queue.submit(&[&buffer]);

        SubmittedCommandBuffer {
            buffer: ManuallyDrop::new(buffer),
            fence: Some(fence),
        }
    }

    pub unsafe fn consume(mut self) -> CommandBuffer {
        let buffer = ManuallyDrop::take(&mut self.buffer).unwrap();
        self.buffer = ManuallyDrop::new(None);

        buffer
    }
}

impl Drop for RecordedCommandBuffer {
    fn drop(&mut self) {
        let buffer = unsafe { ManuallyDrop::take(&mut self.buffer) };
        match buffer {
            Some(_) => {
                warn!("Command buffer was recorded but never submitted.");
            }
            None => {}
        }
    }
}

#[must_use]
pub struct SubmittedCommandBuffer {
    buffer: ManuallyDrop<CommandBuffer>,
    fence: Option<Fence>,
}

impl SubmittedCommandBuffer {
    pub fn wait(&mut self) {
        match &self.fence {
            Some(fence) => {
                fence.wait();
                self.fence = None;
            }
            None => warn!("Duplicate wait on command buffer"),
        }
    }

    pub fn wait_and_reset(mut self) -> CommandBuffer {
        self.wait();
        let buffer = unsafe { ManuallyDrop::take(&mut self.buffer) };
        buffer
    }
}

impl Drop for SubmittedCommandBuffer {
    fn drop(&mut self) {
        match &self.fence {
            Some(fence) => {
                warn!("Submitted command buffer dropped before being waited upon - forcing wait");
                fence.wait();
                unsafe {
                    ManuallyDrop::drop(&mut self.buffer);
                }
            }
            None => (),
        }
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
                .raw()
                .cmd_bind_pipeline(self.buffer.raw, bind_point, pipeline.raw())
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
            self.device().raw().cmd_bind_descriptor_sets(
                self.buffer.raw,
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
            self.device().raw().cmd_pipeline_barrier(
                self.buffer.raw,
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
            self.device().raw().cmd_copy_image(
                self.buffer.raw,
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
                .ext()
                .acceleration_structure
                .cmd_build_acceleration_structures(
                    self.buffer.raw,
                    &[description.geometry_info()],
                    &[description.ranges()],
                );
        }
    }

    pub fn trace_rays(
        &self,
        binding_tables: &RayTracingShaderBindingTables,
        size: (u32, u32, u32),
    ) {
        // info!("binding tables: {:?}", binding_tables);
        unsafe {
            self.device().ext().ray_tracing_pipeline.cmd_trace_rays(
                self.buffer.raw,
                binding_tables.raygen.raw(),
                binding_tables.miss.raw(),
                binding_tables.hit.raw(),
                binding_tables.callable.raw(),
                size.0,
                size.1,
                size.2,
            )
        }
    }
}
