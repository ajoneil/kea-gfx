use ash::khr;
use std::os::raw::c_char;

#[derive(Default)]
pub struct DeviceExtensions {
    pub swapchain: Option<khr::swapchain::Device>,
    pub acceleration_structure: Option<khr::acceleration_structure::Device>,
    pub deferred_host_operations: Option<khr::deferred_host_operations::Device>,
    pub ray_tracing_pipeline: Option<khr::ray_tracing_pipeline::Device>,
}

#[derive(Debug)]
pub enum Ext {
    Swapchain,
    AccelerationStructure,
    DeferredHostOperations,
    RayTracingPipeline,
    RayTracingPositionFetch,
}

impl Ext {
    pub fn name(&self) -> *const c_char {
        match self {
            Ext::Swapchain => khr::swapchain::NAME.as_ptr(),
            Ext::AccelerationStructure => khr::acceleration_structure::NAME.as_ptr(),
            Ext::DeferredHostOperations => khr::deferred_host_operations::NAME.as_ptr(),
            Ext::RayTracingPipeline => khr::ray_tracing_pipeline::NAME.as_ptr(),
            Ext::RayTracingPositionFetch => khr::ray_tracing_position_fetch::NAME.as_ptr(),
        }
    }
}

impl DeviceExtensions {
    pub fn swapchain(&self) -> &khr::swapchain::Device {
        self.swapchain.as_ref().unwrap()
    }

    pub fn acceleration_structure(&self) -> &khr::acceleration_structure::Device {
        self.acceleration_structure.as_ref().unwrap()
    }

    pub fn deferred_host_operations(&self) -> &khr::deferred_host_operations::Device {
        self.deferred_host_operations.as_ref().unwrap()
    }

    pub fn ray_tracing_pipeline(&self) -> &khr::ray_tracing_pipeline::Device {
        self.ray_tracing_pipeline.as_ref().unwrap()
    }

    pub fn new(
        device: &ash::Device,
        instance: &ash::Instance,
        extensions: &[Ext],
    ) -> DeviceExtensions {
        let mut ext = DeviceExtensions {
            ..Default::default()
        };

        for extension in extensions {
            match extension {
                Ext::Swapchain => {
                    ext.swapchain = Some(khr::swapchain::Device::new(instance, device))
                }
                Ext::AccelerationStructure => {
                    ext.acceleration_structure =
                        Some(khr::acceleration_structure::Device::new(instance, device))
                }
                Ext::DeferredHostOperations => {
                    ext.deferred_host_operations =
                        Some(khr::deferred_host_operations::Device::new(instance, device))
                }
                Ext::RayTracingPipeline => {
                    ext.ray_tracing_pipeline =
                        Some(khr::ray_tracing_pipeline::Device::new(instance, device))
                }
                Ext::RayTracingPositionFetch => {}
            }
        }

        ext
    }
}
