use super::queue_family::QueueFamily;
use crate::instance::vulkan_instance::VulkanInstance;
use ash::vk;
use std::{ffi::CStr, fmt, sync::Arc};

pub struct PhysicalDevice {
    vulkan: Arc<VulkanInstance>,
    raw: vk::PhysicalDevice,
    name: String,
}

impl PhysicalDevice {
    pub unsafe fn from_raw(raw: vk::PhysicalDevice, vulkan: Arc<VulkanInstance>) -> PhysicalDevice {
        let props = vulkan.raw().get_physical_device_properties(raw);
        let name = CStr::from_ptr(props.device_name.as_ptr())
            .to_str()
            .unwrap()
            .to_string();

        PhysicalDevice { raw, vulkan, name }
    }

    pub fn queue_families(self: &Arc<Self>) -> Vec<QueueFamily> {
        unsafe {
            self.vulkan
                .raw()
                .get_physical_device_queue_family_properties(self.raw)
        }
        .into_iter()
        .enumerate()
        .map(|(index, properties)| QueueFamily::new(self.clone(), index as u32, properties))
        .collect()
    }

    pub unsafe fn raw(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn vulkan(&self) -> &Arc<VulkanInstance> {
        &self.vulkan
    }
}

impl fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevice({})", self.name)
    }
}
