use std::sync::Arc;

use ash::vk;

use super::Device;

pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,

    pub device: Arc<Device>,
}

impl Swapchain {
    pub fn new(device: &Arc<Device>) -> Swapchain {
        let surface_capabilities = device.surface_capabilities();

        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };

        let surface_format = device.surface_formats()[0];

        let present_mode = device
            .surface_present_modes()
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(device.surface.surface)
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

        let images = unsafe { device.ext.swapchain.get_swapchain_images(swapchain) }.unwrap();
        let image_views =
            Self::create_swapchain_image_views(&images, surface_format.format, &device);

        Swapchain {
            swapchain,
            format: surface_format.format,
            images,
            image_views,

            device: device.clone(),
        }
    }

    fn create_swapchain_image_views(
        swapchain_images: &[vk::Image],
        format: vk::Format,
        device: &Device,
    ) -> Vec<vk::ImageView> {
        swapchain_images
            .iter()
            .map(|&image| {
                let imageview_create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.vk().create_image_view(&imageview_create_info, None) }.unwrap()
            })
            .collect()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.image_views.iter() {
                self.device.vk().destroy_image_view(image_view, None);
            }

            self.device
                .ext
                .swapchain
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
