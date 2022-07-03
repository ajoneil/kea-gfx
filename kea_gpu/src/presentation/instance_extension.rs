use super::Surface;
use crate::{
    device::{PhysicalDevice, QueueFamily},
    instance::{InstanceExtension, VulkanInstance},
};
use ash::{extensions::khr, vk};

pub struct SurfaceExt(khr::Surface);

impl InstanceExtension for SurfaceExt {}

impl SurfaceExt {
    pub fn new(instance: &VulkanInstance) -> Self {
        unsafe {
            let raw = khr::Surface::new(instance.entry(), instance.raw());

            Self(raw)
        }
    }

    pub fn surface_capabilities(
        &self,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.0
                .get_physical_device_surface_capabilities(physical_device.raw(), surface.raw())
        }
        .unwrap()
    }

    pub fn surface_formats(
        &self,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.0
                .get_physical_device_surface_formats(physical_device.raw(), surface.raw())
        }
        .unwrap()
    }

    pub fn surface_present_modes(
        &self,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.0
                .get_physical_device_surface_present_modes(physical_device.raw(), surface.raw())
        }
        .unwrap()
    }

    pub fn surface_support(
        &self,
        physical_device: &PhysicalDevice,
        queue_family: &QueueFamily,
        surface: &Surface,
    ) -> bool {
        unsafe {
            self.0.get_physical_device_surface_support(
                physical_device.raw(),
                queue_family.index(),
                surface.raw(),
            )
        }
        .unwrap()
    }

    pub unsafe fn destroy_surface(&self, surface: &Surface) {
        self.0.destroy_surface(surface.raw(), None)
    }
}
