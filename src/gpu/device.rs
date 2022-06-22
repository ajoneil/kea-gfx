use super::{
    physical_device::{DeviceSelection, QueueFamily},
    surface::Surface,
    vulkan::Vulkan,
};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use log::info;
use std::{
    collections::HashMap,
    iter,
    mem::ManuallyDrop,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Queues {
    pub graphics: Queue,
    pub compute: Queue,
    pub transfer: Queue,
}

#[derive(Clone, Debug)]
pub struct Queue {
    vk: vk::Queue,
    family: QueueFamily,
    index: u32,
}

impl Queue {
    pub unsafe fn vk(&self) -> vk::Queue {
        self.vk
    }

    pub fn family(&self) -> &QueueFamily {
        &self.family
    }
}

pub struct Device {
    device: ash::Device,
    pub queues: Queues,
    pub ext: Extensions,
    pub vulkan: Arc<Vulkan>,
    surface: Surface,
    pub allocator: ManuallyDrop<Mutex<Allocator>>,
}

pub struct Extensions {
    pub swapchain: ash::extensions::khr::Swapchain,
    pub acceleration_structure: ash::extensions::khr::AccelerationStructure,
}

impl Device {
    pub fn new(
        vulkan: Arc<Vulkan>,
        device_selection: DeviceSelection,
        surface: Surface,
    ) -> Arc<Device> {
        let (device, queues) = Self::create_logical_device_with_queue(&vulkan, &device_selection);

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
            physical_device: unsafe { device_selection.physical_device.vk() },
            debug_settings: Default::default(),
            buffer_device_address: true,
        })
        .unwrap();

        Arc::new(Device {
            device,
            surface,
            queues,
            ext,

            vulkan,
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
        })
    }

    fn create_logical_device_with_queue(
        vulkan: &Vulkan,
        device_selection: &DeviceSelection,
    ) -> (ash::Device, Queues) {
        // This code is a mess, but it _should_ create a queue for each purpose
        // in the correct family. Will probably explode if there aren't enough
        // queues. I'm sure there's some iterator magic that could clean this all
        // up.
        let queue_families = [
            &device_selection.graphics,
            &device_selection.compute,
            &device_selection.transfer,
        ];

        let queues: HashMap<u32, Vec<f32>> = queue_families
            .iter()
            .map(|qf| qf.index())
            .fold(HashMap::new(), |mut counts, index| {
                *counts.entry(index).or_insert(0) += 1;
                counts
            })
            .iter()
            .map(|(index, count)| (*index, iter::repeat(1.0).take(*count).collect()))
            .collect();

        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = queues
            .iter()
            .map(|(family_index, priorities)| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*family_index)
                    .queue_priorities(priorities)
                    .build()
            })
            .collect();
        let extension_names = Self::device_extension_names();

        let mut features_12 = vk::PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .vulkan_memory_model(true)
            .build();

        let mut features_13 = vk::PhysicalDeviceVulkan13Features::builder().dynamic_rendering(true);
        let mut features_rt =
            vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder().ray_tracing_pipeline(true);
        let mut features_as = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
            .acceleration_structure(true);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extension_names)
            .push_next(&mut features_12)
            .push_next(&mut features_13)
            .push_next(&mut features_rt)
            .push_next(&mut features_as);

        let device = unsafe {
            vulkan
                .instance
                .create_device(device_selection.physical_device.vk(), &create_info, None)
        }
        .unwrap();
        let families = device_selection.physical_device.queue_families();

        let queues = Queues {
            graphics: Self::queue(
                &device,
                families[device_selection.graphics.index() as usize].clone(),
                0,
            ),
            compute: Self::queue(
                &device,
                families[device_selection.compute.index() as usize].clone(),
                if device_selection.graphics.index() == device_selection.compute.index() {
                    1
                } else {
                    0
                },
            ),
            transfer: Self::queue(
                &device,
                families[device_selection.transfer.index() as usize].clone(),
                [
                    device_selection.graphics.index(),
                    device_selection.compute.index(),
                ]
                .iter()
                .filter(|i| **i == device_selection.transfer.index())
                .count() as u32,
            ),
        };

        info!("Created queues: {:?}", queues);

        (device, queues)
    }

    fn queue(device: &ash::Device, family: QueueFamily, index: u32) -> Queue {
        Queue {
            vk: unsafe { device.get_device_queue(family.index(), index) },
            family,
            index,
        }
    }

    fn device_extension_names() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::AccelerationStructure::name().as_ptr(),
            ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
        ]
    }

    pub unsafe fn vk(&self) -> &ash::Device {
        &self.device
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn queue_wait_idle(&self, queue: &Queue) {
        unsafe {
            self.device.queue_wait_idle(queue.vk()).unwrap();
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            // We need to use manually drop here to ensure the allocator
            // cleans up any remaining memory before the device is destroyed
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
        }
    }
}
