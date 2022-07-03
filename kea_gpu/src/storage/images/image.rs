use super::unallocated_image::UnallocatedImage;
use crate::device::Device;
use ash::vk;
use gpu_allocator::{vulkan::Allocation, MemoryLocation};
use std::{mem::ManuallyDrop, sync::Arc};

pub struct Image {
    name: String,
    image: UnallocatedImage,
    allocation: ManuallyDrop<Allocation>,
    location: MemoryLocation,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        size: (u32, u32),
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        name: String,
        location: MemoryLocation,
    ) -> Self {
        UnallocatedImage::new(device, size, format, usage).allocate(name, location)
    }

    pub unsafe fn from_bound_allocation(
        name: String,
        image: UnallocatedImage,
        allocation: Allocation,
        location: MemoryLocation,
    ) -> Self {
        Self {
            name,
            image,
            allocation: ManuallyDrop::new(allocation),
            location,
        }
    }

    pub unsafe fn raw(&self) -> vk::Image {
        self.image.raw()
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        log::debug!("Freeing {:?}", self.name);
        unsafe {
            self.image
                .device()
                .allocator()
                .lock()
                .unwrap()
                .free(ManuallyDrop::take(&mut self.allocation))
                .unwrap();
        }
    }
}
