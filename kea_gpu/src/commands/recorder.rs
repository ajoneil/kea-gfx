use super::CommandBuffer;
use crate::{
    descriptors::DescriptorSet,
    device::Device,
    pipelines::{Pipeline, PipelineLayout},
};
use ash::vk;
use std::sync::Arc;

pub struct CommandBufferRecorder<'a> {
    buffer: &'a CommandBuffer,
}

impl<'a> CommandBufferRecorder<'a> {
    pub fn new(buffer: &'a CommandBuffer) -> Self {
        Self { buffer }
    }

    pub fn device(&self) -> &Arc<Device> {
        self.buffer.device()
    }

    pub fn bind_pipeline(&self, bind_point: vk::PipelineBindPoint, pipeline: &Pipeline) {
        unsafe {
            self.device()
                .raw()
                .cmd_bind_pipeline(self.buffer.raw(), bind_point, pipeline.raw())
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
                self.buffer.raw(),
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
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier2],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier2],
        image_memory_barriers: &[vk::ImageMemoryBarrier2],
    ) {
        let dependency_info = vk::DependencyInfo::builder()
            .dependency_flags(dependency_flags)
            .memory_barriers(memory_barriers)
            .buffer_memory_barriers(buffer_memory_barriers)
            .image_memory_barriers(image_memory_barriers);
        unsafe {
            self.device()
                .raw()
                .cmd_pipeline_barrier2(self.buffer.raw(), &dependency_info);
        }
    }

    pub unsafe fn buffer(&self) -> &CommandBuffer {
        &self.buffer
    }
}
