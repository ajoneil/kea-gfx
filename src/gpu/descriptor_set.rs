use super::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct DescriptorSetLayout {
    device: Arc<Device>,
    raw: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Arc<Device>,
        bindings: &[DescriptorSetLayoutBinding],
    ) -> DescriptorSetLayout {
        let bindings: Vec<vk::DescriptorSetLayoutBinding> =
            bindings.iter().map(|b| b.raw).collect();
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let raw = unsafe { device.vk().create_descriptor_set_layout(&create_info, None) }.unwrap();

        DescriptorSetLayout { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::DescriptorSetLayout {
        self.raw
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk()
                .destroy_descriptor_set_layout(self.raw, None)
        }
    }
}

pub struct DescriptorSetLayoutBinding {
    raw: vk::DescriptorSetLayoutBinding,
}

impl DescriptorSetLayoutBinding {
    pub fn new(
        binding: u32,
        descriptor_type: vk::DescriptorType,
        descriptor_count: u32,
        stage_flags: vk::ShaderStageFlags,
    ) -> DescriptorSetLayoutBinding {
        let raw = vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(descriptor_count)
            .stage_flags(stage_flags)
            .build();
        DescriptorSetLayoutBinding { raw }
    }
}
