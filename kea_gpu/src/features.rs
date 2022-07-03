use crate::{
    device::{self, DeviceConfig},
    instance::{self, InstanceConfig, InstanceExtension, VulkanInstance},
};

pub trait Feature {
    fn instance_extension_names(&self) -> Vec<instance::Ext> {
        vec![]
    }

    fn instance_extensions(&self, _instance: &VulkanInstance) -> Vec<Box<dyn InstanceExtension>> {
        vec![]
    }

    fn layers(&self) -> Vec<String> {
        vec![]
    }

    fn device_extensions(&self) -> Vec<device::Ext> {
        vec![]
    }

    fn configure_device(&self, _config: &mut DeviceConfig) {}

    fn configure_instance(&self, _config: &mut InstanceConfig) {}
}
