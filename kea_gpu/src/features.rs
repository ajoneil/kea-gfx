use crate::{
    device::{self, DeviceConfig},
    instance::{self, config::InstanceConfig},
};

pub trait Feature {
    fn instance_extensions(&self) -> Vec<instance::Ext> {
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
