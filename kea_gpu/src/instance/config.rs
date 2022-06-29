use crate::debug::features::ValidationFeaturesInstanceConfig;

#[derive(Default, Debug)]
pub struct InstanceConfig {
    pub validation_features: Option<ValidationFeaturesInstanceConfig>,
}
