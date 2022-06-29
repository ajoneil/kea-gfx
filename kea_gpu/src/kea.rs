use crate::{
    core::surface::Surface,
    device::{Device, PhysicalDevice, QueueFamily},
    features::Feature,
    instance::vulkan_instance::VulkanInstance,
    presenter::Presenter,
    surfaces::feature::SurfaceFeature,
    swapchain::feature::SwapchainFeature,
    window::Window,
};
use log::{debug, info};
use std::sync::Arc;

pub struct Kea {
    device: Arc<Device>,
    presenter: Presenter,
}

impl Kea {
    pub fn new(window: &Window, size: (u32, u32), mut features: Vec<Box<dyn Feature + '_>>) -> Kea {
        let mut required_features: Vec<Box<dyn Feature + '_>> = vec![
            Box::new(SurfaceFeature::new()),
            Box::new(SwapchainFeature::new()),
        ];
        required_features.append(&mut features);

        let vulkan = VulkanInstance::new(&required_features);
        let window_surface = Surface::from_window(vulkan.clone(), &window);
        let (physical_device, queue_family) = device_supporting_surface(&vulkan, &window_surface);
        let device = Device::new(
            physical_device.clone(),
            &[(queue_family, 1 as usize)],
            &required_features,
        );
        let presenter = Presenter::new(&device, window_surface, size);

        Kea { device, presenter }
    }

    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        self.device.physical_device()
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn presenter(&self) -> &Presenter {
        &self.presenter
    }
}

fn device_supporting_surface(
    vulkan: &Arc<VulkanInstance>,
    surface: &Surface,
) -> (Arc<PhysicalDevice>, QueueFamily) {
    let physical_devices = vulkan.physical_devices();
    debug!("All devices: {:?}", physical_devices);

    let (physical_device, queue_family) = physical_devices
        .into_iter()
        .filter_map(|physical_device| {
            let queue_family = physical_device.queue_families().into_iter().find(|family| {
                family.supports_graphics()
                    && family.supports_surface(surface)
                    && family.queue_count() >= 1
            });

            match queue_family {
                Some(queue_family) => Some((physical_device, queue_family)),
                None => None,
            }
        })
        .nth(0)
        .unwrap();

    info!("Selected device: {:?}", physical_device);

    (physical_device, queue_family)
}
