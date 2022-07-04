use super::{CommandBufferRecorder, CommandPool};
use crate::{core::sync::Fence, device::Device};
use ash::vk;
use std::{mem::ManuallyDrop, sync::Arc};

pub struct CommandBuffer {
    name: String,
    pool: Arc<CommandPool>,
    raw: vk::CommandBuffer,
}

impl CommandBuffer {
    pub unsafe fn new(name: String, pool: Arc<CommandPool>, raw: vk::CommandBuffer) -> Self {
        Self { name, pool, raw }
    }

    pub fn now<F>(device: &Arc<Device>, name: String, func: F)
    where
        F: FnOnce(&CommandBufferRecorder),
    {
        CommandPool::new(device.graphics_queue())
            .allocate_buffer(name)
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

    fn begin(&self) {
        // log::debug!("{}: recording", self.name);
        unsafe {
            self.device()
                .raw()
                .begin_command_buffer(self.raw, &vk::CommandBufferBeginInfo::default())
        }
        .unwrap();
    }

    fn end(&self) {
        // log::debug!("{}: recording complete", self.name);
        unsafe { self.device().raw().end_command_buffer(self.raw) }.unwrap()
    }

    pub fn device(&self) -> &Arc<Device> {
        self.pool.device()
    }

    pub unsafe fn raw(&self) -> vk::CommandBuffer {
        self.raw
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub unsafe fn raw(&self) -> vk::CommandBuffer {
        self.buffer.as_ref().unwrap().raw()
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
    pub fn wait(mut self) {
        log::debug!("Waiting upon command {}", self.buffer.name());
        self.fence.as_ref().unwrap().wait();
        self.fence = None;
        log::debug!("Command {} complete", self.buffer.name());
    }

    pub fn wait_and_reuse(mut self) -> RecordedCommandBuffer {
        log::debug!("Waiting upon command {}", self.buffer.name());
        self.fence.as_ref().unwrap().wait();
        self.fence = None;
        log::debug!("Command {} complete", self.buffer.name());

        RecordedCommandBuffer {
            buffer: unsafe { ManuallyDrop::new(Some(ManuallyDrop::take(&mut self.buffer))) },
        }
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
                log::debug!("Command {} complete", self.buffer.name());

                unsafe {
                    ManuallyDrop::drop(&mut self.buffer);
                }
            }
            None => (),
        }
    }
}
