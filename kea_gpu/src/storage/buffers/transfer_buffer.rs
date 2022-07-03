use super::{AlignedBuffer, Buffer};
use crate::{commands::CommandBuffer, device::Device};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub struct TransferBuffer {
    device: Arc<Device>,
    name: String,
    cpu_buffer: Buffer,
    usage: vk::BufferUsageFlags,
}

impl TransferBuffer {
    pub fn new(
        device: Arc<Device>,
        size: u64,
        usage: vk::BufferUsageFlags,
        name: String,
    ) -> TransferBuffer {
        let cpu_buffer = Buffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            format!("{} transfer", name),
            MemoryLocation::CpuToGpu,
        );

        TransferBuffer {
            device,
            name,
            cpu_buffer,
            usage,
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
        );

        CommandBuffer::now(&self.device, |cmd| {
            cmd.copy_buffer(&self.cpu_buffer, &gpu_buffer)
        });

        gpu_buffer
    }

    pub fn transfer_to_gpu_with_alignment(&mut self, alignment: u32) -> AlignedBuffer {
        let usage = self.usage | vk::BufferUsageFlags::TRANSFER_DST;
        let gpu_buffer = AlignedBuffer::new(
            self.device.clone(),
            self.cpu_buffer().size() as _,
            alignment,
            usage,
            self.name.clone(),
        );

        CommandBuffer::now(&self.device, |cmd| {
            cmd.copy_buffer_aligned(&self.cpu_buffer, &gpu_buffer)
        });

        gpu_buffer
    }
}
