use crate::{features::Feature, instance};

pub struct SurfaceFeature {}

impl SurfaceFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for SurfaceFeature {
    fn instance_extensions(&self) -> Vec<instance::Ext> {
        vec![instance::Ext::Surface, instance::Ext::WaylandSurface]
    }
}
