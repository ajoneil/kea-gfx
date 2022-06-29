use crate::{device, features::Feature};

pub struct SwapchainFeature {}

impl SwapchainFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for SwapchainFeature {
    fn device_extensions(&self) -> Vec<device::Ext> {
        vec![device::Ext::Swapchain]
    }
}
