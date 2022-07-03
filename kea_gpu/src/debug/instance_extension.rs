use super::messenger::DebugMessenger;
use crate::instance::{InstanceExtension, VulkanInstance};
use ash::extensions::ext;

pub struct DebugUtilsExt {
    _raw: ext::DebugUtils,
    _messenger: DebugMessenger,
}

impl InstanceExtension for DebugUtilsExt {}

impl DebugUtilsExt {
    pub fn new(instance: &VulkanInstance) -> Self {
        unsafe {
            let raw = ext::DebugUtils::new(instance.entry(), instance.raw());
            let messenger = DebugMessenger::new(&raw);

            Self {
                _raw: raw,
                _messenger: messenger,
            }
        }
    }
}
