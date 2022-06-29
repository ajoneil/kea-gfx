use std::os::raw::c_char;

use ash::{extensions::khr, vk};

#[derive(Debug)]
pub enum Ext {
    Surface,
    WaylandSurface,
    ValidationFeatures,
}

impl Ext {
    pub fn name(&self) -> *const c_char {
        match self {
            Ext::Surface => khr::Surface::name().as_ptr(),
            Ext::WaylandSurface => khr::WaylandSurface::name().as_ptr(),
            Ext::ValidationFeatures => vk::ExtValidationFeaturesFn::name().as_ptr(),
        }
    }
}

#[derive(Default)]
pub struct InstanceExtensions {
    pub surface: Option<khr::Surface>,
    pub wayland_surface: Option<khr::WaylandSurface>,
}

impl InstanceExtensions {
    pub fn surface(&self) -> &khr::Surface {
        self.surface.as_ref().unwrap()
    }

    pub fn wayland_surface(&self) -> &khr::WaylandSurface {
        self.wayland_surface.as_ref().unwrap()
    }

    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        extensions: &[Ext],
    ) -> InstanceExtensions {
        let mut ext = InstanceExtensions {
            ..Default::default()
        };

        for extension in extensions {
            match extension {
                Ext::Surface => ext.surface = Some(khr::Surface::new(entry, instance)),
                Ext::WaylandSurface => {
                    ext.wayland_surface = Some(khr::WaylandSurface::new(entry, instance))
                }
                Ext::ValidationFeatures => (),
            }
        }

        ext
    }
}
