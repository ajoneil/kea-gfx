use std::{
    ffi::CStr,
    mem::ManuallyDrop,
    sync::{Arc, Mutex},
};

use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use log::info;

use super::{Surface, Vulkan};

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub ext: Extensions,
    pub vulkan: Arc<Vulkan>,
    pub surface: Surface,
    pub allocator: ManuallyDrop<Mutex<Allocator>>,
}

pub struct Extensions {
    pub swapchain: ash::extensions::khr::Swapchain,
    pub acceleration_structure: ash::extensions::khr::AccelerationStructure,
}

impl Device {
    pub fn new(vulkan: &Arc<Vulkan>, surface: Surface) -> Device {
        let (physical_device, queue_family_index) = Self::select_physical_device(vulkan, &surface);

        let (device, queue) =
            Self::create_logical_device_with_queue(vulkan, physical_device, queue_family_index);

        let ext = Extensions {
            swapchain: ash::extensions::khr::Swapchain::new(&vulkan.instance, &device),
            acceleration_structure: ash::extensions::khr::AccelerationStructure::new(
                &vulkan.instance,
                &device,
            ),
        };

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: vulkan.instance.clone(),
            device: device.clone(),
            physical_device: physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
        })
        .unwrap();

        Device {
            physical_device,
            device,
            surface,
            queue,
            queue_family_index,
            ext,

            vulkan: vulkan.clone(),
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
        }
    }

    fn select_physical_device(vulkan: &Vulkan, surface: &Surface) -> (vk::PhysicalDevice, u32) {
        let devices = unsafe { vulkan.instance.enumerate_physical_devices() }.unwrap();
        let (device, queue_family_index) = devices
            .into_iter()
            .find_map(
                |device| match Self::find_queue_family_index(&vulkan, device, surface) {
                    Some(index) => Some((device, index)),
                    None => None,
                },
            )
            .unwrap();

        let props = unsafe { vulkan.instance.get_physical_device_properties(device) };

        info!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        (device, queue_family_index)
    }

    fn find_queue_family_index(
        vulkan: &Vulkan,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> Option<u32> {
        let props = unsafe {
            vulkan
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };
        props
            .iter()
            .enumerate()
            .find(|(index, family)| {
                family.queue_count > 0
                    && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && unsafe {
                        vulkan.ext.surface.get_physical_device_surface_support(
                            physical_device,
                            *index as u32,
                            surface.surface,
                        )
                    }
                    .unwrap()
            })
            .map(|(index, _)| index as _)
    }

    fn create_logical_device_with_queue(
        vulkan: &Vulkan,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> (ash::Device, vk::Queue) {
        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])
            .build()];
        let extension_names = Self::device_extension_names();

        let mut features_12 = vk::PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .vulkan_memory_model(true)
            .build();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extension_names)
            .push_next(&mut features_12);

        let device = unsafe {
            vulkan
                .instance
                .create_device(physical_device, &create_info, None)
        }
        .unwrap();
        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        (device, present_queue)
    }

    fn device_extension_names() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::AccelerationStructure::name().as_ptr(),
        ]
    }

    pub fn surface_capabilities(&self) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_capabilities(
                    self.physical_device,
                    self.surface.surface,
                )
        }
        .unwrap()
    }

    pub fn surface_formats(&self) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_formats(self.physical_device, self.surface.surface)
        }
        .unwrap()
    }

    pub fn surface_present_modes(&self) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.vulkan
                .ext
                .surface
                .get_physical_device_surface_present_modes(
                    self.physical_device,
                    self.surface.surface,
                )
        }
        .unwrap()
    }

    pub unsafe fn vk(&self) -> &ash::Device {
        &self.device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            // We need to use manually drop here to ensure the allocator
            // cleans up any remaining memory before the device is destroyed
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
        }
    }
}
