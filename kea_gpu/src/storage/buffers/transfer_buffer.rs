use super::Buffer;
use crate::{commands::CommandBuffer, device::Device};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub struct TransferBuffer {
    device: Arc<Device>,
    name: String,
    cpu_buffer: Buffer,
    usage: vk::BufferUsageFlags,
    alignment: Option<u64>,
}

impl TransferBuffer {
    pub fn new(
        device: Arc<Device>,
        size: u64,
        usage: vk::BufferUsageFlags,
        name: String,
        alignment: Option<u64>,
    ) -> TransferBuffer {
        let cpu_buffer = Buffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            format!("{} transfer", name),
            MemoryLocation::CpuToGpu,
            None,
        );

        TransferBuffer {
            device,
            name,
            cpu_buffer,
            usage,
            alignment,
        }
    }

    pub fn cpu_buffer(&mut self) -> &mut Buffer {
        &mut self.cpu_buffer
    }

    pub fn transfer_to_gpu(&mut self) -> Buffer {
        let usage = self.usage | vk::BufferUsageFlags::TRANSFER_DST;
        let gpu_buffer = Buffer::new(
            self.device.clone(),
            self.cpu_buffer().size() as _,
            usage,
            self.name.clone(),
            MemoryLocation::GpuOnly,
            self.alignment,
        );

        CommandBuffer::now(
            &self.device,
            format!("transfer {} to gpu", self.name),
            |cmd| cmd.copy_buffer(&self.cpu_buffer, &gpu_buffer),
        );

        gpu_buffer
    }
}
