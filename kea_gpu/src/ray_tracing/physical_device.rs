use crate::device::PhysicalDevice;
use ash::vk;

impl PhysicalDevice {
    pub fn ray_tracing_pipeline_properties(
        &self,
    ) -> vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
        let mut rt_props = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::builder().build();
        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut rt_props)
            .build();

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
        let mut accel_props =
            vk::PhysicalDeviceAccelerationStructurePropertiesKHR::builder().build();
        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut accel_props)
            .build();

        unsafe {
            self.instance()
                .raw()
                .get_physical_device_properties2(self.raw(), &mut props)
        }

        accel_props
    }
}
