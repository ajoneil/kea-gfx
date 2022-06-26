use ash::vk;
use std::{
    ffi::CStr,
    fmt::{self},
    sync::Arc,
};

use crate::gpu::{surface::Surface, vulkan::VulkanInstance};

use super::queue_family::QueueFamily;

pub struct PhysicalDevice {
    vulkan: Arc<VulkanInstance>,
    raw: vk::PhysicalDevice,
    name: String,
}

impl PhysicalDevice {
    pub unsafe fn from_raw(raw: vk::PhysicalDevice, vulkan: Arc<VulkanInstance>) -> PhysicalDevice {
        let props = vulkan.raw().get_physical_device_properties(raw);
        let name = CStr::from_ptr(props.device_name.as_ptr())
            .to_str()
            .unwrap()
            .to_string();

        PhysicalDevice { raw, vulkan, name }
    }

    pub fn queue_families(self: &Arc<Self>) -> Vec<QueueFamily> {
        unsafe {
            self.vulkan
                .raw()
                .get_physical_device_queue_family_properties(self.raw)
        }
        .into_iter()
        .enumerate()
        .map(|(index, properties)| QueueFamily::new(self.clone(), index as u32, properties))
        .collect()
    }

    pub unsafe fn raw(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn vulkan(&self) -> &Arc<VulkanInstance> {
        &self.vulkan
    }

    pub fn surface_capabilities(&self, surface: &Surface) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.vulkan
                .ext()
                .surface
                .get_physical_device_surface_capabilities(self.raw, surface.raw())
        }
        .unwrap()
    }

    pub fn surface_formats(&self, surface: &Surface) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.vulkan
                .ext()
                .surface
                .get_physical_device_surface_formats(self.raw, surface.raw())
        }
        .unwrap()
    }

    pub fn surface_present_modes(&self, surface: &Surface) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.vulkan
                .ext()
                .surface
                .get_physical_device_surface_present_modes(self.raw, surface.raw())
        }
        .unwrap()
    }

    pub fn ray_tracing_pipeline_properties(
        &self,
    ) -> vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
        let mut rt_props = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::builder().build();
        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut rt_props)
            .build();

        unsafe {
            self.vulkan
                .raw()
                .get_physical_device_properties2(self.raw, &mut props)
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
            self.vulkan
                .raw()
                .get_physical_device_properties2(self.raw, &mut props)
        }

        accel_props
    }
}

impl fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevice({})", self.name)
    }
}
