use super::{CommandBufferRecorder, CommandPool};
use crate::{core::sync::Fence, device::Device};
use ash::vk;
use std::{mem::ManuallyDrop, sync::Arc};

pub struct CommandBuffer {
    pool: Arc<CommandPool>,
    raw: vk::CommandBuffer,
}

impl CommandBuffer {
    pub unsafe fn new(pool: Arc<CommandPool>, raw: vk::CommandBuffer) -> Self {
        Self { pool, raw }
    }

    pub fn now<F>(device: &Arc<Device>, func: F)
    where
        F: FnOnce(&CommandBufferRecorder),
    {
        CommandPool::new(device.graphics_queue())
            .allocate_buffer()
            .record(func)
            .submit()
            .wait();
    }

    pub fn record<F>(self, func: F) -> RecordedCommandBuffer
    where
        F: FnOnce(&CommandBufferRecorder),
    {
        self.begin();
        func(&CommandBufferRecorder::new(&self));
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
        let fence = buffer.pool.queue().submit(&[&buffer]);

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
                log::warn!("Command buffer was recorded but never submitted.");
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
            None => log::warn!("Duplicate wait on command buffer"),
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
                log::warn!(
                    "Submitted command buffer dropped before being waited upon - forcing wait"
                );
                fence.wait();
                unsafe {
                    ManuallyDrop::drop(&mut self.buffer);
                }
            }
            None => (),
        }
    }
}
