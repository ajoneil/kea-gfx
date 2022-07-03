use crate::debug::DebugInstanceConfig;

#[derive(Default, Debug)]
pub struct InstanceConfig {
    pub validation_features: Option<DebugInstanceConfig>,
}
