use ash::vk;

use crate::{
    features::Feature,
    instance::{self, config::InstanceConfig},
};

#[derive(Debug)]
pub struct ValidationFeaturesInstanceConfig {
    pub enable: Vec<vk::ValidationFeatureEnableEXT>,
    pub disable: Vec<vk::ValidationFeatureDisableEXT>,
}

pub struct VulkanValidationFeature {}

impl Feature for VulkanValidationFeature {
    fn instance_extensions(&self) -> Vec<instance::Ext> {
        vec![instance::Ext::ValidationFeatures]
    }

    fn layers(&self) -> Vec<String> {
        vec![String::from("VK_LAYER_KHRONOS_validation")]
    }

    fn configure_instance(&self, config: &mut InstanceConfig) {
        config.validation_features = Some(ValidationFeaturesInstanceConfig {
            enable: vec![
                vk::ValidationFeatureEnableEXT::GPU_ASSISTED,
                vk::ValidationFeatureEnableEXT::GPU_ASSISTED_RESERVE_BINDING_SLOT,
                vk::ValidationFeatureEnableEXT::BEST_PRACTICES,
                vk::ValidationFeatureEnableEXT::DEBUG_PRINTF,
                vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
            ],
            disable: vec![],
        })
    }
}

impl VulkanValidationFeature {
    pub fn new() -> Self {
        Self {}
    }
}
