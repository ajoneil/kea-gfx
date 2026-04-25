use ash::vk;
use downcast_rs::{impl_downcast, Downcast};
use std::os::raw::c_char;

#[derive(Debug)]
pub enum Ext {
    Surface,
    WaylandSurface,
    XlibSurface,
    LayerSettings,
    DebugUtils,
}

impl Ext {
    pub fn name(&self) -> *const c_char {
        match self {
            Ext::Surface => vk::KHR_SURFACE_NAME.as_ptr(),
            Ext::WaylandSurface => vk::KHR_WAYLAND_SURFACE_NAME.as_ptr(),
            Ext::XlibSurface => vk::KHR_XLIB_SURFACE_NAME.as_ptr(),
            Ext::LayerSettings => vk::EXT_LAYER_SETTINGS_NAME.as_ptr(),
            Ext::DebugUtils => vk::EXT_DEBUG_UTILS_NAME.as_ptr(),
        }
    }
}

pub trait InstanceExtension: Downcast {}
impl_downcast!(InstanceExtension);
