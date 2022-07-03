use super::Image;
use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct ImageView {
    image: Arc<Image>,
    raw: vk::ImageView,
}

impl ImageView {
    pub fn new(image: Arc<Image>) -> ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(unsafe { image.raw() })
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format())
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

        let raw = unsafe {
            image
                .device()
                .raw()
                .create_image_view(&imageview_create_info, None)
        }
        .unwrap();

        ImageView { image, raw }
    }

    pub unsafe fn raw(&self) -> vk::ImageView {
        self.raw
    }

    pub fn device(&self) -> &Arc<Device> {
        self.image.device()
    }

    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device().raw().destroy_image_view(self.raw(), None);
        }
    }
}
