use super::CommandBuffer;
use crate::{device::Device, queues::Queue};
use ash::vk;
use std::{slice, sync::Arc};

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

    pub fn allocate_buffers(self: &Arc<Self>, names: &[String]) -> Vec<CommandBuffer> {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.raw)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(names.len() as _);

        let raws = unsafe { self.device().raw().allocate_command_buffers(&create_info) }.unwrap();

        names
            .into_iter()
            .zip(raws)
            .map(|(name, raw)| unsafe { CommandBuffer::new(name.to_string(), self.clone(), raw) })
            .collect()
    }

    pub fn allocate_buffer(self: &Arc<Self>, name: String) -> CommandBuffer {
        self.allocate_buffers(slice::from_ref(&name))
            .into_iter()
            .nth(0)
            .unwrap()
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
