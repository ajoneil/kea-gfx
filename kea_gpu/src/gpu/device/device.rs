use super::{queue_family::QueueFamily, PhysicalDevice};
use crate::gpu::{command::CommandBuffer, sync::Fence};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::{
    iter,
    mem::ManuallyDrop,
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
        let buffers: Vec<vk::CommandBuffer> =
            command_buffers.into_iter().map(|cmd| cmd.buffer).collect();
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
    ext: Extensions,
    allocator: ManuallyDrop<Mutex<Allocator>>,
    queues: Vec<QueueHandle>,
}

pub struct Extensions {
    pub swapchain: ash::extensions::khr::Swapchain,
    pub acceleration_structure: ash::extensions::khr::AccelerationStructure,
    pub deferred_host_operations: ash::extensions::khr::DeferredHostOperations,
    pub ray_tracing_pipeline: ash::extensions::khr::RayTracingPipeline,
}

impl Device {
    pub fn new(
        physical_device: Arc<PhysicalDevice>,
        queues: &[(QueueFamily, usize)],
    ) -> Arc<Device> {
        let raw = create_device(&physical_device, queues);
        let vulkan = physical_device.vulkan();

        let ext = unsafe {
            Extensions {
                swapchain: ash::extensions::khr::Swapchain::new(&vulkan.raw(), &raw),
                acceleration_structure: ash::extensions::khr::AccelerationStructure::new(
                    &vulkan.raw(),
                    &raw,
                ),
                deferred_host_operations: ash::extensions::khr::DeferredHostOperations::new(
                    &vulkan.raw(),
                    &raw,
                ),
                ray_tracing_pipeline: ash::extensions::khr::RayTracingPipeline::new(
                    &vulkan.raw(),
                    &raw,
                ),
            }
        };

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

    fn device_extension_names() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::AccelerationStructure::name().as_ptr(),
            ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
            ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
        ]
    }

    pub fn wait_until_idle(&self) {
        unsafe {
            self.raw.device_wait_idle().unwrap();
        }
    }

    pub unsafe fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub unsafe fn ext(&self) -> &Extensions {
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

fn create_device(physical_device: &PhysicalDevice, queues: &[(QueueFamily, usize)]) -> ash::Device {
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
    let extension_names = Device::device_extension_names();

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

    device
}
