use std::sync::Arc;

use crate::{
    gpu::{
        device::Device, physical_device::PhysicalDevice, surface::Surface, swapchain::Swapchain,
        vulkan::VulkanInstance,
    },
    presenter::Presenter,
    window::Window,
};

pub struct Kea {
    vulkan: Arc<VulkanInstance>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    presenter: Presenter,
}

impl Kea {
    pub fn new(window: &Window) -> Kea {
        let vulkan = VulkanInstance::new(window.required_extensions());
        let window_surface = Surface::from_window(vulkan.clone(), &window);
        let device_selection = PhysicalDevice::select_physical_device(&vulkan, &window_surface);
        let device = Device::new(vulkan.clone(), device_selection.clone(), window_surface);
        let swapchain = Swapchain::new(&device, &device_selection.physical_device);
        let presenter = Presenter::new(swapchain);

        Kea {
            vulkan,
            physical_device: Arc::new(device_selection.physical_device),
            device,
            presenter,
        }
    }

    pub fn vulkan(&self) -> &Arc<VulkanInstance> {
        &self.vulkan
    }

    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn presenter(&self) -> &Presenter {
        &self.presenter
    }
}
