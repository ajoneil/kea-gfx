use super::CommandBuffer;
use crate::device::{Device, Queue};
use ash::vk;
use std::sync::Arc;

pub struct CommandPool {
    queue: Queue,
    raw: vk::CommandPool,
}

impl CommandPool {
    pub fn new(queue: Queue) -> Arc<CommandPool> {
        let create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(queue.family().index());
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
            .map(|raw| unsafe { CommandBuffer::new(self.clone(), raw) })
            .collect()
    }

    pub fn allocate_buffer(self: &Arc<Self>) -> CommandBuffer {
        self.allocate_buffers(1).into_iter().nth(0).unwrap()
    }

    pub fn device(&self) -> &Arc<Device> {
        self.queue.device()
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
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
