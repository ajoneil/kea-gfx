use ash::extensions::khr;
use std::os::raw::c_char;

#[derive(Default)]
pub struct DeviceExtensions {
    pub swapchain: Option<khr::Swapchain>,
    pub acceleration_structure: Option<khr::AccelerationStructure>,
    pub deferred_host_operations: Option<khr::DeferredHostOperations>,
    pub ray_tracing_pipeline: Option<khr::RayTracingPipeline>,
}

#[derive(Debug)]
pub enum Ext {
    Swapchain,
    AccelerationStructure,
    DeferredHostOperations,
    RayTracingPipeline,
}

impl Ext {
    pub fn name(&self) -> *const c_char {
        match self {
            Ext::Swapchain => khr::Swapchain::name().as_ptr(),
            Ext::AccelerationStructure => khr::AccelerationStructure::name().as_ptr(),
            Ext::DeferredHostOperations => khr::DeferredHostOperations::name().as_ptr(),
            Ext::RayTracingPipeline => khr::RayTracingPipeline::name().as_ptr(),
        }
    }
}

impl DeviceExtensions {
    pub fn swapchain(&self) -> &ash::extensions::khr::Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    pub fn acceleration_structure(&self) -> &khr::AccelerationStructure {
        self.acceleration_structure.as_ref().unwrap()
    }

    pub fn deferred_host_operations(&self) -> &khr::DeferredHostOperations {
        self.deferred_host_operations.as_ref().unwrap()
    }

    pub fn ray_tracing_pipeline(&self) -> &khr::RayTracingPipeline {
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
                Ext::Swapchain => ext.swapchain = Some(khr::Swapchain::new(instance, device)),
                Ext::AccelerationStructure => {
                    ext.acceleration_structure =
                        Some(khr::AccelerationStructure::new(instance, device))
                }
                Ext::DeferredHostOperations => {
                    ext.deferred_host_operations =
                        Some(khr::DeferredHostOperations::new(instance, device))
                }
                Ext::RayTracingPipeline => {
                    ext.ray_tracing_pipeline = Some(khr::RayTracingPipeline::new(instance, device))
                }
            }
        }

        ext
    }
}
