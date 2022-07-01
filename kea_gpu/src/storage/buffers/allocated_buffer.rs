use super::Buffer;
use crate::device::Device;
use ash::vk;
use gpu_allocator::vulkan::Allocation;
use std::{
    mem::{self, ManuallyDrop},
    slice,
};

pub struct AllocatedBuffer {
    name: String,
    buffer: Buffer,
    allocation: ManuallyDrop<Allocation>,
}

impl AllocatedBuffer {
    pub fn new(name: String, buffer: Buffer, allocation: Allocation) -> Self {
        Self {
            name,
            buffer,
            allocation: ManuallyDrop::new(allocation),
        }
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        unsafe {
            self.buffer.device().raw().get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder().buffer(self.buffer.raw()),
            )
        }
    }

    pub fn allocated_size(&self) -> usize {
        self.allocation.size() as usize
    }

    pub fn fill<T>(&self, data: &[T]) {
        let data = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<T>())
        };
        assert!(data.len() == self.buffer.size());

        unsafe {
            let pointer = self.allocation.mapped_ptr().unwrap().as_ptr();
            pointer.copy_from_nonoverlapping(data.as_ptr() as _, data.len());
            // let mut align = ash::util::Align::new(
            //     pointer,
            //     mem::align_of::<T>() as _,
            //     mem::size_of_val(data) as _,
            // );
            // align.copy_from_slice(data);
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn device(&self) -> &Device {
        self.buffer.device()
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        log::debug!("Freeing {:?}", self.name);
        unsafe {
            self.buffer
                .device()
                .allocator()
                .lock()
                .unwrap()
                .free(ManuallyDrop::take(&mut self.allocation))
                .unwrap();
        }
    }
}
