use super::{
    extensions::{DeviceExtensions, Ext},
    physical_device::PhysicalDevice,
    QueueFamily,
};
use crate::{
    core::{command::CommandBuffer, sync::Fence},
    features::Feature,
};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use log::info;
use std::{
    iter,
    mem::ManuallyDrop,
    os::raw::c_char,
    sync::{Arc, Mutex},
};

pub struct Queue {
    device: Arc<Device>,
    raw: vk::Queue,
    family: QueueFamily,
}

impl Queue {
    pub unsafe fn raw(&self) -> vk::Queue {
        self.raw
    }

    pub fn family(&self) -> &QueueFamily {
        &self.family
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn submit(&self, command_buffers: &[&CommandBuffer]) -> Fence {
        let fence = Fence::new(self.device.clone(), false);
        let buffers: Vec<vk::CommandBuffer> = command_buffers
            .into_iter()
            .map(|cmd| unsafe { cmd.raw() })
            .collect();
        let submits = [vk::SubmitInfo::builder().command_buffers(&buffers).build()];
        unsafe {
            self.device
                .raw()
                .queue_submit(self.raw(), &submits, fence.vk())
                .unwrap();
        }

        fence
    }
}

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
        let (raw, extensions) = create_device(&physical_device, queues, features);
        let vulkan = physical_device.vulkan();
        let ext = DeviceExtensions::new(&raw, unsafe { vulkan.raw() }, &extensions);

        let allocator = unsafe {
            Allocator::new(&AllocatorCreateDesc {
                instance: vulkan.raw().clone(),
                device: raw.clone(),
                physical_device: physical_device.raw(),
                debug_settings: Default::default(),
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

        Queue {
            device: self.clone(),
            family: handle.family.clone(),
            raw: handle.raw,
        }
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

fn create_device(
    physical_device: &PhysicalDevice,
    queues: &[(QueueFamily, usize)],
    features: &[Box<dyn Feature + '_>],
) -> (ash::Device, Vec<Ext>) {
    // Priorities vec needs to exist on the stack to prevent the optimiser deleting
    // it before we use it (.build() throws away lifetimes)
    let queues_with_priorities: Vec<(u32, Vec<f32>)> = queues
        .iter()
        .map(|(family, count)| {
            let priorities = iter::repeat(1.0).take(*count).collect();
            (family.index(), priorities)
        })
        .collect();

    let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = queues_with_priorities
        .iter()
        .map(|(family_index, priorities)| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*family_index)
                .queue_priorities(priorities)
                .build()
        })
        .collect();

    let mut extensions: Vec<Ext> = vec![];
    for feature in features {
        for ext in feature.device_extensions() {
            extensions.push(ext);
        }
    }
    let extension_names: Vec<*const c_char> = extensions.iter().map(|ext| ext.name()).collect();
    info!("Requested device extensions: {:?}", extensions);

    let mut features_12 = vk::PhysicalDeviceVulkan12Features::builder()
        .buffer_device_address(true)
        .vulkan_memory_model(true)
        .build();

    let mut features_13 = vk::PhysicalDeviceVulkan13Features::builder().dynamic_rendering(true);
    let mut features_rt =
        vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder().ray_tracing_pipeline(true);
    let mut features_as =
        vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder().acceleration_structure(true);

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&extension_names)
        .push_next(&mut features_12)
        .push_next(&mut features_13)
        .push_next(&mut features_rt)
        .push_next(&mut features_as);

    let device = unsafe {
        physical_device
            .vulkan()
            .raw()
            .create_device(physical_device.raw(), &create_info, None)
    }
    .unwrap();

    (device, extensions)
}
