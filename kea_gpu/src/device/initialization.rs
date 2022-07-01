use std::{iter, os::raw::c_char};
use ash::vk;
use log::info;
use crate::features::Feature;
use super::{Ext, PhysicalDevice, QueueFamily};

#[derive(Default, Debug)]
pub struct DeviceConfig {}

pub fn create_device(
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

    let mut device_config = DeviceConfig::default();

    let mut extensions: Vec<Ext> = vec![];
    for feature in features {
        for ext in feature.device_extensions() {
            extensions.push(ext);
        }

        feature.configure_device(&mut device_config);
    }

    info!("Device configuration: {:?}", device_config);

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

// Workaround missing ash binding
struct ValidationFeatures(vk::ValidationFeaturesEXT);

unsafe impl vk::ExtendsDeviceCreateInfo for ValidationFeatures {}
