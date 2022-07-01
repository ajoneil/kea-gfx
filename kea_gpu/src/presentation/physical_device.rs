use super::Surface;
use crate::device::PhysicalDevice;
use ash::vk;

impl PhysicalDevice {
    pub fn surface_capabilities(&self, surface: &Surface) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.vulkan()
                .ext()
                .surface()
                .get_physical_device_surface_capabilities(self.raw(), surface.raw())
        }
        .unwrap()
    }

    pub fn surface_formats(&self, surface: &Surface) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.vulkan()
                .ext()
                .surface()
                .get_physical_device_surface_formats(self.raw(), surface.raw())
        }
        .unwrap()
    }

    pub fn surface_present_modes(&self, surface: &Surface) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.vulkan()
                .ext()
                .surface()
                .get_physical_device_surface_present_modes(self.raw(), surface.raw())
        }
        .unwrap()
    }
}
