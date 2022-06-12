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
        let pool = unsafe { device.device.create_command_pool(&create_info, None) }.unwrap();

        CommandPool { pool, device }
    }

    pub fn allocate_buffer(self: Arc<Self>) -> CommandBuffer {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let buffer =
            unsafe { self.device.device.allocate_command_buffers(&create_info) }.unwrap()[0];

        CommandBuffer {
            buffer,
            _pool: self,
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_command_pool(self.pool, None);
        }
    }
}

pub struct CommandBuffer {
    pub buffer: vk::CommandBuffer,
    _pool: Arc<CommandPool>,
}
