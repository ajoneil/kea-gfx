use crate::{descriptors::DescriptorSetLayout, device::Device};
use ash::vk;
use std::{slice, sync::Arc};

pub struct PipelineLayout {
    device: Arc<Device>,
    raw: vk::PipelineLayout,
    descriptor_set_layout: DescriptorSetLayout,
}

impl PipelineLayout {
    pub fn new(device: Arc<Device>, descriptor_set_layout: DescriptorSetLayout) -> PipelineLayout {
        let layout_raw = unsafe { descriptor_set_layout.raw() };
        let create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(slice::from_ref(&layout_raw));

        let raw = unsafe { device.raw().create_pipeline_layout(&create_info, None) }.unwrap();

        PipelineLayout {
            device,
            raw,
            descriptor_set_layout,
        }
    }

    pub unsafe fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }

    pub fn descriptor_set_layout(&self) -> &DescriptorSetLayout {
        &self.descriptor_set_layout
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.raw, None);
        }
    }
}
