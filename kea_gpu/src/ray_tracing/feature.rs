use crate::{ device, features::Feature};

pub struct RayTracingFeature {}

impl RayTracingFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for RayTracingFeature {
    fn device_extensions(&self) -> Vec<device::Ext> {
        vec![
            device::Ext::AccelerationStructure,
            device::Ext::DeferredHostOperations,
            device::Ext::RayTracingPipeline,
        ]
    }
}