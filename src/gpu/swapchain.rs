use std::sync::Arc;

use ash::vk;

use super::{Device, Surface};

pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,

    device: Arc<Device>,
}

impl Swapchain {
    pub fn new(device: &Arc<Device>, surface: &Surface) -> Swapchain {
        let surface_capabilities = unsafe {
            device
                .vulkan
                .ext
                .surface
                .get_physical_device_surface_capabilities(device.physical_device, surface.surface)
        }
        .unwrap();

        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };

        let surface_format = unsafe {
            device
                .vulkan
                .ext
                .surface
                .get_physical_device_surface_formats(device.physical_device, surface.surface)
        }
        .unwrap()[0];

        let present_mode = unsafe {
            device
                .vulkan
                .ext
                .surface
                .get_physical_device_surface_present_modes(device.physical_device, surface.surface)
        }
        .unwrap()
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(vk::Extent2D {
                width: 1920,
                height: 1080,
            })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .present_mode(present_mode);

        let swapchain = unsafe {
            device
                .ext
                .swapchain
                .create_swapchain(&swapchain_create_info, None)
        }
        .unwrap();

        Swapchain {
            swapchain,
            format: surface_format.format,

            device: device.clone(),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ext
                .swapchain
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
