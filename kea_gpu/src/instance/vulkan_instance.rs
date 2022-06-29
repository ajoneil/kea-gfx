use ash::vk;
use log::info;
use std::{ffi::CString, os::raw::c_char, sync::Arc};

use crate::{device::PhysicalDevice, features::Feature, instance::config::InstanceConfig};

use super::{extensions::InstanceExtensions, Ext};

pub struct VulkanInstance {
    entry: ash::Entry,
    raw: ash::Instance,
    ext: InstanceExtensions,
}

impl VulkanInstance {
    pub fn new(features: &[Box<dyn Feature + '_>]) -> Arc<VulkanInstance> {
        let entry = ash::Entry::linked();
        let (raw, extensions) = Self::create_instance(&entry, features);
        let ext = InstanceExtensions::new(&entry, &raw, &extensions);
        Arc::new(VulkanInstance { entry, raw, ext })
    }

    fn create_instance(
        entry: &ash::Entry,
        features: &[Box<dyn Feature + '_>],
    ) -> (ash::Instance, Vec<Ext>) {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let mut instance_config = InstanceConfig::default();
        let mut extensions: Vec<Ext> = vec![];
        let mut layers: Vec<CString> = vec![];

        for feature in features {
            for ext in feature.instance_extensions() {
                extensions.push(ext);
            }

            for layer in feature.layers() {
                layers.push(CString::new(layer).unwrap());
            }

            feature.configure_instance(&mut instance_config);
        }
        let extension_names: Vec<*const c_char> = extensions.iter().map(|ext| ext.name()).collect();
        info!("Instance config: {:?}", instance_config);
        info!("Requested instance extensions: {:?}", extensions);
        info!("Requested layers: {:?}", layers);

        let layers_names_raw: Vec<*const c_char> =
            layers.iter().map(|raw_name| raw_name.as_ptr()).collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_names_raw);

        let mut features_validation = vk::ValidationFeaturesEXT {
            ..Default::default()
        };
        let create_info = if let Some(_) = instance_config.validation_features {
            let enable = &instance_config.validation_features.as_ref().unwrap().enable;
            let disable = &instance_config
                .validation_features
                .as_ref()
                .unwrap()
                .disable;
            features_validation = vk::ValidationFeaturesEXT::builder()
                .enabled_validation_features(enable)
                .disabled_validation_features(disable)
                .build();

            create_info.push_next(&mut features_validation)
        } else {
            create_info
        };
        let create_info = create_info.build();

        log::debug!("{:?}", create_info);

        let raw = unsafe { entry.create_instance(&create_info, None).unwrap() };

        (raw, extensions)
    }

    pub unsafe fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub unsafe fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub unsafe fn ext(&self) -> &InstanceExtensions {
        &self.ext
    }

    pub fn physical_devices(self: &Arc<VulkanInstance>) -> Vec<Arc<PhysicalDevice>> {
        unsafe {
            self.raw
                .enumerate_physical_devices()
                .unwrap()
                .into_iter()
                .map(|physical_device: vk::PhysicalDevice| {
                    Arc::new(PhysicalDevice::from_raw(physical_device, self.clone()))
                })
                .collect()
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.raw.destroy_instance(None) };
    }
}
