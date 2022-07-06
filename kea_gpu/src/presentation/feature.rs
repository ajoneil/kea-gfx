use super::SurfaceExt;
use crate::{
    device,
    features::Feature,
    instance::{self, InstanceExtension, VulkanInstance},
};

pub struct PresentationFeature {}

impl PresentationFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for PresentationFeature {
    fn instance_extension_names(&self) -> Vec<instance::Ext> {
        vec![
            instance::Ext::Surface,
            instance::Ext::WaylandSurface,
            instance::Ext::XlibSurface,
        ]
    }

    fn instance_extensions(&self, instance: &VulkanInstance) -> Vec<Box<dyn InstanceExtension>> {
        vec![Box::new(SurfaceExt::new(instance))]
    }

    fn device_extensions(&self) -> Vec<crate::device::Ext> {
        vec![device::Ext::Swapchain]
    }

    fn layers(&self) -> Vec<String> {
        vec![]
    }

    fn configure_device(&self, _config: &mut device::DeviceConfig) {}

    fn configure_instance(&self, _config: &mut instance::InstanceConfig) {}
}
