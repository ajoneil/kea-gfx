use super::{Surface, SurfaceExt};
use crate::{
    core::sync::Semaphore,
    device::Device,
    storage::images::{Image, ImageOwnership, ImageView},
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

pub struct Swapchain {
    device: Arc<Device>,
    _surface: Surface,
    raw: vk::SwapchainKHR,
    extent: vk::Extent2D,
    format: vk::Format,
    images: Vec<ImageView>,
}

impl Swapchain {
    pub fn new(device: &Arc<Device>, surface: Surface, extent: vk::Extent2D) -> Swapchain {
        let surface_capabilities = device
            .instance()
            .ext::<SurfaceExt>()
            .surface_capabilities(device.physical_device(), &surface);

        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };

        let available_formats = device
            .instance()
            .ext::<SurfaceExt>()
            .surface_formats(device.physical_device(), &surface);
        let surface_format = available_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&available_formats[0]);

        let present_mode = device
            .instance()
            .ext::<SurfaceExt>()
            .surface_present_modes(device.physical_device(), &surface)
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(unsafe { surface.raw() })
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .present_mode(present_mode);

        let raw = unsafe {
            device
                .ext()
                .swapchain()
                .create_swapchain(&swapchain_create_info, None)
        }
        .unwrap();

        let images = Self::create_images(
            device,
            raw,
            (extent.width, extent.height),
            surface_format.format,
        );

        Swapchain {
            raw,
            _surface: surface,
            format: surface_format.format,
            images,
            extent,
            device: device.clone(),
        }
    }

    fn create_images(
        device: &Arc<Device>,
        swapchain: vk::SwapchainKHR,
        size: (u32, u32),
        format: vk::Format,
    ) -> Vec<ImageView> {
        unsafe { device.ext().swapchain().get_swapchain_images(swapchain) }
            .unwrap()
            .into_iter()
            .map(|raw| unsafe {
                let image = Image::from_raw(
                    device.clone(),
                    raw,
                    "Swapchain image".to_string(),
                    size,
                    format,
                    MemoryLocation::GpuOnly,
                    ImageOwnership::ExternallyOwned,
                );
                ImageView::new(Arc::new(image))
            })
            .collect()
    }

    pub fn acquire_next_image(&self, semaphore: &Semaphore) -> (u32, &ImageView) {
        let (image_index, _) = unsafe {
            self.device.ext().swapchain().acquire_next_image(
                self.raw,
                u64::MAX,
                semaphore.vk(),
                vk::Fence::null(),
            )
        }
        .unwrap();

        (image_index, &self.images[image_index as usize])
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub unsafe fn raw(&self) -> vk::SwapchainKHR {
        self.raw
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ext()
                .swapchain()
                .destroy_swapchain(self.raw, None);
        }
    }
}
