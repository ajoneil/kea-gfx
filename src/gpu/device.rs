use super::{
    command::CommandBuffer,
    physical_device::{DeviceSelection, QueueFamily},
    surface::Surface,
    sync::Fence,
    vulkan::Vulkan,
};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use log::info;
use std::{
    collections::HashMap,
    fmt::Debug,
    iter,
    mem::ManuallyDrop,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct Queues {
    graphics: QueueHandle,
    compute: QueueHandle,
    transfer: QueueHandle,
}

pub struct Queue {
    device: Arc<Device>,
    vk: vk::Queue,
    family: QueueFamily,
    index: u32,
}

#[derive(Debug)]
struct QueueHandle {
    vk: vk::Queue,
    family: QueueFamily,
    index: u32,
}

pub struct DeviceQueues<'a> {
    device: &'a Arc<Device>,
    queues: &'a Queues,
}

impl<'a> DeviceQueues<'a> {
    pub fn graphics(&self) -> Queue {
        Queue {
            device: self.device.clone(),
            vk: self.queues.graphics.vk,
            family: self.queues.graphics.family.clone(),
            index: self.queues.graphics.index,
        }
    }
}

impl Queue {
    pub unsafe fn vk(&self) -> vk::Queue {
        self.vk
    }

    pub fn family(&self) -> &QueueFamily {
        &self.family
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn submit(&self, command_buffers: &[&CommandBuffer]) -> Fence {
        let fence = Fence::new(self.device.clone(), false);
        let buffers: Vec<vk::CommandBuffer> =
            command_buffers.into_iter().map(|cmd| cmd.buffer).collect();
        let submits = [vk::SubmitInfo::builder().command_buffers(&buffers).build()];
        unsafe {
            self.device
                .vk()
                .queue_submit(self.vk(), &submits, fence.vk())
                .unwrap();
        }

        fence
    }
}

pub struct Device {
    device: ash::Device,
    queues: Queues,
    surface: Surface,
    pub ext: Extensions,
    pub vulkan: Arc<Vulkan>,
    pub allocator: ManuallyDrop<Mutex<Allocator>>,
}

pub struct Extensions {
    pub swapchain: ash::extensions::khr::Swapchain,
    pub acceleration_structure: ash::extensions::khr::AccelerationStructure,
    pub deferred_host_operations: ash::extensions::khr::DeferredHostOperations,
    pub ray_tracing_pipeline: ash::extensions::khr::RayTracingPipeline,
}

impl Device {
    pub fn new(
        vulkan: Arc<Vulkan>,
        device_selection: DeviceSelection,
        surface: Surface,
    ) -> Arc<Device> {
        let device = Self::create_logical_device(&vulkan, &device_selection);

        let ext = Extensions {
            swapchain: ash::extensions::khr::Swapchain::new(&vulkan.instance, &device),
            acceleration_structure: ash::extensions::khr::AccelerationStructure::new(
                &vulkan.instance,
                &device,
            ),
            deferred_host_operations: ash::extensions::khr::DeferredHostOperations::new(
                &vulkan.instance,
                &device,
            ),
            ray_tracing_pipeline: ash::extensions::khr::RayTracingPipeline::new(
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

        let queues = Self::create_queues(&device_selection, &device);

        Arc::new(Device {
            device,
            surface,
            queues,
            ext,

            vulkan,
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
        })
    }

    fn create_logical_device(vulkan: &Vulkan, device_selection: &DeviceSelection) -> ash::Device {
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

        device
    }

    fn create_queues(device_selection: &DeviceSelection, vk_device: &ash::Device) -> Queues {
        let families = device_selection.physical_device.queue_families();

        let queues = Queues {
            graphics: Self::queue(
                vk_device,
                families[device_selection.graphics.index() as usize].clone(),
                0,
            ),
            compute: Self::queue(
                vk_device,
                families[device_selection.compute.index() as usize].clone(),
                if device_selection.graphics.index() == device_selection.compute.index() {
                    1
                } else {
                    0
                },
            ),
            transfer: Self::queue(
                vk_device,
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

        queues
    }

    fn queue(vk_device: &ash::Device, family: QueueFamily, index: u32) -> QueueHandle {
        QueueHandle {
            vk: unsafe { vk_device.get_device_queue(family.index(), index) },
            family,
            index,
        }
    }

    fn device_extension_names() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::AccelerationStructure::name().as_ptr(),
            ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
            ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
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

    pub fn queues<'a>(self: &'a Arc<Self>) -> DeviceQueues<'a> {
        DeviceQueues {
            device: self,
            queues: &self.queues,
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
