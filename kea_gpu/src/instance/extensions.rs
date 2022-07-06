use ash::{
    extensions::{ext, khr},
    vk,
};
use downcast_rs::{impl_downcast, Downcast};
use std::os::raw::c_char;

#[derive(Debug)]
pub enum Ext {
    Surface,
    WaylandSurface,
    XlibSurface,
    ValidationFeatures,
    DebugUtils,
}

impl Ext {
    pub fn name(&self) -> *const c_char {
        match self {
            Ext::Surface => khr::Surface::name().as_ptr(),
            Ext::WaylandSurface => khr::WaylandSurface::name().as_ptr(),
            Ext::XlibSurface => khr::XlibSurface::name().as_ptr(),
            Ext::ValidationFeatures => vk::ExtValidationFeaturesFn::name().as_ptr(),
            Ext::DebugUtils => ext::DebugUtils::name().as_ptr(),
        }
    }
}

pub trait InstanceExtension: Downcast {}
impl_downcast!(InstanceExtension);
