use super::messenger::DebugMessenger;
use crate::instance::{InstanceExtension, VulkanInstance};
use ash::ext;

pub struct DebugUtilsExt {
    _raw: ext::debug_utils::Instance,
    _messenger: DebugMessenger,
}

impl InstanceExtension for DebugUtilsExt {}

impl DebugUtilsExt {
    pub fn new(instance: &VulkanInstance) -> Self {
        unsafe {
            let raw = ext::debug_utils::Instance::new(instance.entry(), instance.raw());
            let messenger = DebugMessenger::new(&raw);

            Self {
                _raw: raw,
                _messenger: messenger,
            }
        }
    }
}
