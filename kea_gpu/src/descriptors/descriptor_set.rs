use crate::device::Device;
use ash::vk;
use std::sync::Arc;

pub struct DescriptorSetLayout {
    device: Arc<Device>,
    raw: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Arc<Device>,
        bindings: &[DescriptorSetLayoutBinding<'_>],
    ) -> DescriptorSetLayout {
        let bindings: Vec<vk::DescriptorSetLayoutBinding<'_>> =
            bindings.iter().map(|b| b.raw).collect();
        let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let raw = unsafe {
            device
                .raw()
                .create_descriptor_set_layout(&create_info, None)
        }
        .unwrap();

        DescriptorSetLayout { device, raw }
    }

    pub unsafe fn raw(&self) -> vk::DescriptorSetLayout {
        self.raw
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw()
                .destroy_descriptor_set_layout(self.raw, None)
        }
    }
}

pub struct DescriptorSetLayoutBinding<'a> {
    raw: vk::DescriptorSetLayoutBinding<'a>,
}

impl<'a> DescriptorSetLayoutBinding<'a> {
    pub fn new(
        binding: u32,
        descriptor_type: vk::DescriptorType,
        descriptor_count: u32,
        stage_flags: vk::ShaderStageFlags,
    ) -> DescriptorSetLayoutBinding<'a> {
        let raw = vk::DescriptorSetLayoutBinding::default()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(descriptor_count)
            .stage_flags(stage_flags);
        DescriptorSetLayoutBinding { raw }
    }
}

pub struct DescriptorPool {
    device: Arc<Device>,
    raw: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(
        device: Arc<Device>,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> Arc<DescriptorPool> {
        let create_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(pool_sizes);
        let raw = unsafe { device.raw().create_descriptor_pool(&create_info, None) }.unwrap();

        Arc::new(DescriptorPool { device, raw })
    }

    pub fn allocate_descriptor_sets(
        self: &Arc<Self>,
        layouts: &[DescriptorSetLayout],
    ) -> Vec<DescriptorSet> {
        let raw_layouts: Vec<vk::DescriptorSetLayout> = layouts
            .iter()
            .map(|layout| unsafe { layout.raw() })
            .collect();
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.raw)
            .set_layouts(&raw_layouts);

        let descriptor_sets =
            unsafe { self.device.raw().allocate_descriptor_sets(&allocate_info) }.unwrap();

        descriptor_sets
            .into_iter()
            .map(|raw| DescriptorSet {
                pool: self.clone(),
                raw,
            })
            .collect()
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_descriptor_pool(self.raw, None);
        }
    }
}

pub struct DescriptorSet {
    pool: Arc<DescriptorPool>,
    raw: vk::DescriptorSet,
}

impl DescriptorSet {
    pub unsafe fn raw(&self) -> vk::DescriptorSet {
        self.raw
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.pool.device()
    }
}
