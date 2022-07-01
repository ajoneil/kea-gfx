use crate::{device, features::Feature, instance};

pub struct PresentationFeature {}

impl PresentationFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for PresentationFeature {
    fn instance_extensions(&self) -> Vec<instance::Ext> {
        vec![instance::Ext::Surface, instance::Ext::WaylandSurface]
    }

    fn device_extensions(&self) -> Vec<crate::device::Ext> {
        vec![device::Ext::Swapchain]
    }
}
