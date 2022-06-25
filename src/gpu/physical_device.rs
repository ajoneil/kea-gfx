use super::{surface::Surface, vulkan::Vulkan};
use ash::vk;
use log::info;
use std::ffi::CStr;

#[derive(Clone)]
pub struct PhysicalDevice<'a> {
    vk: vk::PhysicalDevice,
    vulkan: &'a Vulkan,
    name: String,
}

#[derive(Clone)]
pub struct DeviceSelection<'a> {
    pub physical_device: PhysicalDevice<'a>,
    pub graphics: QueueFamily,
    pub compute: QueueFamily,
    pub transfer: QueueFamily,
}

impl<'a> PhysicalDevice<'a> {
    pub fn new(vk: vk::PhysicalDevice, vulkan: &Vulkan) -> PhysicalDevice {
        let props = unsafe { vulkan.instance.get_physical_device_properties(vk) };
        let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();

        PhysicalDevice { vk, vulkan, name }
    }

    pub fn select_physical_device<'b>(
        vulkan: &'b Vulkan,
        surface: &Surface,
    ) -> DeviceSelection<'b> {
        let devices = vulkan.physical_devices();
        let device_selection = devices
            .into_iter()
            .find_map(|physical_device: PhysicalDevice| {
                let queue_families = physical_device.queue_families();

                let gfx_family = queue_families.iter().find_map(|family: &QueueFamily| {
                    if family.capabilities().contains(&QueueCapability::Graphics)
                        && family.queue_count() > 0
                        && physical_device.queue_family_supports_surface(&family, surface)
                    {
                        Some(family)
                    } else {
                        None
                    }
                });

                let compute_family = queue_families
                    .iter()
                    .filter(|family| {
                        family.capabilities().contains(&QueueCapability::Compute)
                            && family.queue_count() > 0
                    })
                    .max_by_key(|family| {
                        -(family
                            .capabilities()
                            .iter()
                            .filter(|cap| **cap != QueueCapability::Compute)
                            .count() as i32)
                    });

                let transfer_family = queue_families
                    .iter()
                    .filter(|family| {
                        family.capabilities().contains(&QueueCapability::Transfer)
                            && family.queue_count() > 0
                    })
                    .max_by_key(|family| {
                        -(family
                            .capabilities()
                            .into_iter()
                            .filter(|cap| **cap != QueueCapability::Transfer)
                            .count() as i32)
                    });

                match (gfx_family, compute_family, transfer_family) {
                    (Some(gfx_family), Some(compute_family), Some(transfer_family)) => {
                        Some(DeviceSelection {
                            physical_device,
                            graphics: gfx_family.clone(),
                            compute: compute_family.clone(),
                            transfer: transfer_family.clone(),
                        })
                    }
                    _ => None,
                }
            })
            .unwrap();

        info!(
            "Selected physical device: {:?}",
            device_selection.physical_device.name()
        );
        info!(
            "All queue families: {:?}",
            device_selection.physical_device.queue_families()
        );
        info!("Graphics queue family: {:?}", device_selection.graphics);
        info!("Compute queue family: {:?}", device_selection.compute);
        info!("Transfer queue family: {:?}", device_selection.transfer);

        device_selection
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn queue_families(&self) -> Vec<QueueFamily> {
        unsafe {
            self.vulkan
                .instance
                .get_physical_device_queue_family_properties(self.vk)
        }
        .into_iter()
        .enumerate()
        .map(|(index, properties)| QueueFamily::new(index as u32, properties))
        .collect()
    }

    pub unsafe fn vk(&self) -> vk::PhysicalDevice {
        self.vk
    }

    pub fn surface_capabilities(&self, surface: &Surface) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_capabilities(self.vk, surface.surface)
        }
        .unwrap()
    }

    pub fn surface_formats(&self, surface: &Surface) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_formats(self.vk, surface.surface)
        }
        .unwrap()
    }

    pub fn surface_present_modes(&self, surface: &Surface) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_present_modes(self.vk, surface.surface)
        }
        .unwrap()
    }

    pub fn queue_family_supports_surface(
        &self,
        queue_family: &QueueFamily,
        surface: &Surface,
    ) -> bool {
        unsafe {
            self.vulkan.ext.surface.get_physical_device_surface_support(
                self.vk,
                queue_family.index,
                surface.surface,
            )
        }
        .unwrap()
    }

    pub fn properties(&self) -> vk::PhysicalDeviceProperties2 {
        let mut props = vk::PhysicalDeviceProperties2::builder().build();
        unsafe {
            self.vulkan
                .instance
                .get_physical_device_properties2(self.vk, &mut props)
        }

        props
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
                .instance
                .get_physical_device_properties2(self.vk, &mut props)
        }

        rt_props
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QueueCapability {
    Graphics,
    Compute,
    Transfer,
}

#[derive(Clone, Debug)]
pub struct QueueFamily {
    index: u32,
    queue_count: u32,
    capabilities: Vec<QueueCapability>,
}

impl QueueFamily {
    pub fn new(index: u32, family_properties: vk::QueueFamilyProperties) -> QueueFamily {
        QueueFamily {
            index,
            queue_count: family_properties.queue_count,
            capabilities: Self::capabilities_from_queue_flags(family_properties.queue_flags),
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

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn queue_count(&self) -> u32 {
        self.queue_count
    }

    pub fn capabilities(&self) -> &[QueueCapability] {
        &self.capabilities
    }
}
