use crate::{device, instance};

pub trait Feature {
    fn instance_extensions(&self) -> Vec<instance::Ext> {
        vec![]
    }

    fn device_extensions(&self) -> Vec<device::Ext> {
        vec![]
    }
}
