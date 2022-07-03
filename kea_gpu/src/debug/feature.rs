use super::DebugUtilsExt;
use crate::{
    features::Feature,
    instance::{self, InstanceConfig, InstanceExtension, VulkanInstance},
};
use ash::vk;

#[derive(Debug)]
pub struct DebugInstanceConfig {
    pub enable: Vec<vk::ValidationFeatureEnableEXT>,
    pub disable: Vec<vk::ValidationFeatureDisableEXT>,
}

pub struct DebugFeature {}

impl Feature for DebugFeature {
    fn instance_extension_names(&self) -> Vec<instance::Ext> {
        vec![instance::Ext::ValidationFeatures, instance::Ext::DebugUtils]
    }

    fn instance_extensions(&self, instance: &VulkanInstance) -> Vec<Box<dyn InstanceExtension>> {
        vec![Box::new(DebugUtilsExt::new(instance))]
    }

    fn layers(&self) -> Vec<String> {
        vec![String::from("VK_LAYER_KHRONOS_validation")]
    }

    fn configure_instance(&self, config: &mut InstanceConfig) {
        config.validation_features = Some(DebugInstanceConfig {
            enable: vec![
                vk::ValidationFeatureEnableEXT::GPU_ASSISTED,
                vk::ValidationFeatureEnableEXT::GPU_ASSISTED_RESERVE_BINDING_SLOT,
                vk::ValidationFeatureEnableEXT::BEST_PRACTICES,
                // vk::ValidationFeatureEnableEXT::DEBUG_PRINTF,
                vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
            ],
            disable: vec![],
        })
    }
}

impl DebugFeature {
    pub fn new() -> Self {
        Self {}
    }
}
