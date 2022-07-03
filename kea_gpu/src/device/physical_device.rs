use super::queue_family::QueueFamily;
use crate::instance::VulkanInstance;
use ash::vk;
use std::{ffi::CStr, fmt, sync::Arc};

pub struct PhysicalDevice {
    instance: Arc<VulkanInstance>,
    raw: vk::PhysicalDevice,
    name: String,
}

impl PhysicalDevice {
    pub unsafe fn from_raw(
        raw: vk::PhysicalDevice,
        instance: Arc<VulkanInstance>,
    ) -> PhysicalDevice {
        let props = instance.raw().get_physical_device_properties(raw);
        let name = CStr::from_ptr(props.device_name.as_ptr())
            .to_str()
            .unwrap()
            .to_string();

        PhysicalDevice {
            raw,
            instance,
            name,
        }
    }

    pub fn queue_families(self: &Arc<Self>) -> Vec<QueueFamily> {
        unsafe {
            self.instance
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

    pub fn instance(&self) -> &Arc<VulkanInstance> {
        &self.instance
    }
}

impl fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevice({})", self.name)
    }
}
