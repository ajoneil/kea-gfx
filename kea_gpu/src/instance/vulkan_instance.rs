use super::{Ext, InstanceExtension};
use crate::{device::PhysicalDevice, features::Feature, instance::InstanceConfig};
use ash::vk;
use log::info;
use std::{any::TypeId, collections::HashMap, ffi::CString, os::raw::c_char, sync::Arc};

pub struct VulkanInstance {
    entry: ash::Entry,
    raw: ash::Instance,
    extensions: HashMap<TypeId, Box<dyn InstanceExtension>>,
}

impl VulkanInstance {
    pub fn new(features: &[Box<dyn Feature + '_>]) -> Arc<VulkanInstance> {
        let entry = ash::Entry::linked();
        let raw = Self::create_instance(&entry, features);
        let extensions = HashMap::new();

        let mut instance = VulkanInstance {
            entry,
            raw,
            extensions,
        };
        instance.add_extensions(features);

        Arc::new(instance)
    }

    fn create_instance(entry: &ash::Entry, features: &[Box<dyn Feature + '_>]) -> ash::Instance {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let mut instance_config = InstanceConfig::default();
        let mut extensions: Vec<Ext> = vec![];
        let mut layers: Vec<CString> = vec![];

        for feature in features {
            for ext in feature.instance_extension_names() {
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

        #[allow(unused_assignments)]
        let mut features_validation = vk::ValidationFeaturesEXT::default();
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

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    fn add_extensions(&mut self, features: &[Box<dyn Feature + '_>]) {
        for feature in features {
            for extension in feature.instance_extensions(self) {
                self.extensions
                    .insert(extension.as_ref().type_id(), extension);
            }
        }
    }

    pub unsafe fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub unsafe fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn ext<T: InstanceExtension>(&self) -> &T {
        let ext = self.extensions.get(&TypeId::of::<T>()).unwrap().as_ref();
        ext.downcast_ref::<T>().unwrap()
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
