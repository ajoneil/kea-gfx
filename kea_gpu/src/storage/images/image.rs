use crate::{device::Device, storage::memory::Allocation};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub enum ImageOwnership {
    ExternallyOwned,
    MemoryManaged(Option<Allocation>),
}

pub struct Image {
    device: Arc<Device>,
    raw: vk::Image,
    name: String,
    _size: (u32, u32),
    format: vk::Format,
    location: MemoryLocation,
    ownership: ImageOwnership,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        name: String,
        size: (u32, u32),
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        location: MemoryLocation,
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

        let mut image = Image {
            device,
            raw,
            name,
            _size: size,
            format,
            location,
            ownership: ImageOwnership::MemoryManaged(None),
        };
        image.allocate();

        image
    }

    fn allocate(&mut self) {
        match &self.ownership {
            ImageOwnership::ExternallyOwned => {
                panic!("Can't allocate memory for image that is externally owned")
            }
            ImageOwnership::MemoryManaged(memory) => {
                if memory.is_none() {
                    let requirements =
                        unsafe { self.device.raw().get_image_memory_requirements(self.raw) };

                    let allocation = Allocation::new(
                        self.device.clone(),
                        self.name.clone(),
                        self.location,
                        requirements,
                    );

                    unsafe {
                        self.device
                            .raw()
                            .bind_image_memory(self.raw, allocation.memory(), allocation.offset())
                            .unwrap();
                    }

                    self.ownership = ImageOwnership::MemoryManaged(Some(allocation));
                } else {
                    panic!("Can't allocate memory for image twice");
                }
            }
        }
    }

    pub unsafe fn from_raw(
        device: Arc<Device>,
        raw: vk::Image,
        name: String,
        size: (u32, u32),
        format: vk::Format,
        location: MemoryLocation,
        ownership: ImageOwnership,
    ) -> Self {
        Image {
            device,
            raw,
            name,
            _size: size,
            format,
            location,
            ownership,
        }
    }

    pub unsafe fn raw(&self) -> vk::Image {
        self.raw
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        match &self.ownership {
            &ImageOwnership::MemoryManaged(_) => unsafe {
                // Cleanup any memory usage before the buffer is destroyed
                self.ownership = ImageOwnership::MemoryManaged(None);
                self.device.raw().destroy_image(self.raw, None);
            },
            _ => (),
        }
    }
}
