use super::DebugUtilsExt;
use crate::{
    features::Feature,
    instance::{self, InstanceConfig, InstanceExtension, VulkanInstance},
};
use std::ffi::CString;

/// Validation toggles routed through VK_EXT_layer_settings. Each entry is
/// (setting_name, enabled).
#[derive(Debug)]
pub struct DebugInstanceConfig {
    pub layer_settings: Vec<(CString, bool)>,
}

pub struct DebugFeature {}

impl Feature for DebugFeature {
    fn instance_extension_names(&self) -> Vec<instance::Ext> {
        vec![instance::Ext::LayerSettings, instance::Ext::DebugUtils]
    }

    fn instance_extensions(&self, instance: &VulkanInstance) -> Vec<Box<dyn InstanceExtension>> {
        vec![Box::new(DebugUtilsExt::new(instance))]
    }

    fn layers(&self) -> Vec<String> {
        vec![String::from("VK_LAYER_KHRONOS_validation")]
    }

    fn configure_instance(&self, config: &mut InstanceConfig) {
        config.validation_features = Some(DebugInstanceConfig {
            layer_settings: vec![
                (CString::new("gpuav_enable").unwrap(), true),
                (CString::new("validate_best_practices").unwrap(), true),
                (CString::new("validate_sync").unwrap(), true),
            ],
        });
    }
}

impl DebugFeature {
    pub fn new() -> Self {
        Self {}
    }
}
