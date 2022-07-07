use super::{extensions::DeviceExtensions, physical_device::PhysicalDevice, QueueFamily};
use crate::{features::Feature, instance::VulkanInstance, queues::Queue};
use ash::vk;
use gpu_allocator::{
    vulkan::{Allocator, AllocatorCreateDesc},
    AllocatorDebugSettings,
};
use std::{
    mem::ManuallyDrop,
    sync::{Arc, Mutex},
};

pub struct QueueHandle {
    raw: vk::Queue,
    family: QueueFamily,
}

pub struct Device {
    physical_device: Arc<PhysicalDevice>,
    raw: ash::Device,
    ext: DeviceExtensions,
    allocator: ManuallyDrop<Mutex<Allocator>>,
    queues: Vec<QueueHandle>,
}

impl Device {
    pub fn new(
        physical_device: Arc<PhysicalDevice>,
        queues: &[(QueueFamily, usize)],
        features: &[Box<dyn Feature + '_>],
    ) -> Arc<Device> {
        let (raw, extensions) =
            super::initialization::create_device(&physical_device, queues, features);
        let instance = physical_device.instance();
        let ext = DeviceExtensions::new(&raw, unsafe { instance.raw() }, &extensions);

        let allocator = unsafe {
            Allocator::new(&AllocatorCreateDesc {
                instance: instance.raw().clone(),
                device: raw.clone(),
                physical_device: physical_device.raw(),
                debug_settings: AllocatorDebugSettings {
                    log_memory_information: true,
                    log_leaks_on_shutdown: true,
                    log_allocations: true,
                    log_frees: true,
                    ..Default::default()
                },
                buffer_device_address: true,
            })
        }
        .unwrap();

        let queues = queues
            .iter()
            .map(|(family, count)| {
                (0..*count).map(|index| QueueHandle {
                    raw: unsafe { raw.get_device_queue(family.index(), index as u32) },
                    family: family.clone(),
                })
            })
            .flatten()
            .collect();

        Arc::new(Device {
            physical_device,
            raw,
            ext,
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
            queues,
        })
    }

    pub fn physical_device(&self) -> &Arc<PhysicalDevice> {
        &self.physical_device
    }

    pub fn allocator(&self) -> &Mutex<Allocator> {
        &self.allocator
    }

    pub fn wait_until_idle(&self) {
        unsafe {
            self.raw.device_wait_idle().unwrap();
        }
    }

    pub unsafe fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub unsafe fn ext(&self) -> &DeviceExtensions {
        &self.ext
    }

    pub fn instance(&self) -> &VulkanInstance {
        self.physical_device.instance()
    }

    // pub fn queues(self: &Arc<Self>) -> Vec<Queue> {
    //     self.queues
    //         .iter()
    //         .map(|handle| Queue {
    //             device: self.clone(),
    //             family: handle.family.clone(),
    //             raw: handle.raw,
    //         })
    //         .collect()
    // }

    pub fn graphics_queue(self: &Arc<Self>) -> Queue {
        let handle = self
            .queues
            .iter()
            .find(|handle| handle.family.supports_graphics())
            .unwrap();

        unsafe { Queue::new_from_raw(self.clone(), handle.raw, handle.family.clone()) }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.device_wait_idle().unwrap();

            // We need to use manually drop here to ensure the allocator
            // cleans up any remaining memory before the device is destroyed
            ManuallyDrop::drop(&mut self.allocator);
            self.raw.destroy_device(None);
        }
    }
}
