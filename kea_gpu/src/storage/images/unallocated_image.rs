use crate::device::Device;
use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, MemoryLocation};
use std::sync::Arc;

use super::Image;

pub struct UnallocatedImage {
    device: Arc<Device>,
    raw: vk::Image,
    format: vk::Format,
    size: (u32, u32),
}

impl UnallocatedImage {
    pub fn new(
        device: Arc<Device>,
        size: (u32, u32),
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Self {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: size.0,
                height: size.1,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let raw = unsafe { device.raw().create_image(&image_create_info, None) }.unwrap();

        Self {
            device,
            raw,
            size,
            format,
        }
    }

    pub fn allocate(self, name: String, location: MemoryLocation) -> Image {
        let requirements = unsafe { self.device.raw().get_image_memory_requirements(self.raw) };

        let allocation = self
            .device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: &name,
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        unsafe {
            self.device
                .raw()
                .bind_image_memory(self.raw, allocation.memory(), allocation.offset())
                .unwrap();

            Image::from_bound_allocation(name, self, allocation, location)
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub unsafe fn raw(&self) -> vk::Image {
        self.raw
    }
}

impl Drop for UnallocatedImage {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_image(self.raw, None);
        }
    }
}
