use super::physical_device::PhysicalDevice;
use crate::core::surface::Surface;
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QueueCapability {
    Graphics,
    Compute,
    Transfer,
}

#[derive(Clone, Debug)]
pub struct QueueFamily {
    physical_device: Arc<PhysicalDevice>,
    index: u32,
    queue_count: u32,
    capabilities: Vec<QueueCapability>,
}

impl QueueFamily {
    pub fn new(
        physical_device: Arc<PhysicalDevice>,
        index: u32,
        family_properties: vk::QueueFamilyProperties,
    ) -> QueueFamily {
        QueueFamily {
            physical_device,
            index,
            queue_count: family_properties.queue_count,
            capabilities: capabilities_from_queue_flags(family_properties.queue_flags),
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn queue_count(&self) -> u32 {
        self.queue_count
    }

    pub fn capabilities(&self) -> &[QueueCapability] {
        &self.capabilities
    }

    pub fn supports_capability(&self, capability: QueueCapability) -> bool {
        self.capabilities().iter().any(|cap| *cap == capability)
    }

    pub fn supports_graphics(&self) -> bool {
        self.supports_capability(QueueCapability::Graphics)
    }

    pub fn supports_surface(&self, surface: &Surface) -> bool {
        unsafe {
            self.physical_device
                .vulkan()
                .ext()
                .surface()
                .get_physical_device_surface_support(
                    self.physical_device.raw(),
                    self.index(),
                    surface.raw(),
                )
        }
        .unwrap()
    }
}

fn capabilities_from_queue_flags(queue_flags: vk::QueueFlags) -> Vec<QueueCapability> {
    let mappings = [
        (vk::QueueFlags::GRAPHICS, QueueCapability::Graphics),
        (vk::QueueFlags::COMPUTE, QueueCapability::Compute),
        (vk::QueueFlags::TRANSFER, QueueCapability::Transfer),
    ];

    mappings
        .into_iter()
        .filter_map(|(flag, capability)| {
            if queue_flags.contains(flag) {
                Some(capability)
            } else {
                None
            }
        })
        .collect()
}
