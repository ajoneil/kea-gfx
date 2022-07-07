use crate::device::PhysicalDevice;
use ash::vk;

impl PhysicalDevice {
    pub fn ray_tracing_pipeline_properties(
        &self,
    ) -> vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
        let mut rt_props = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut props = vk::PhysicalDeviceProperties2::builder().push_next(&mut rt_props);

        unsafe {
            self.instance()
                .raw()
                .get_physical_device_properties2(self.raw(), &mut props)
        }

        rt_props
    }

    pub fn acceleration_structure_properties(
        &self,
    ) -> vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
        let mut accel_props = vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();
        let mut props = vk::PhysicalDeviceProperties2::builder().push_next(&mut accel_props);

        unsafe {
            self.instance()
                .raw()
                .get_physical_device_properties2(self.raw(), &mut props)
        }

        accel_props
    }
}
