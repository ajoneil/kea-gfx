use super::{
    acceleration_structures::{
        AccelerationStructure, AccelerationStructureDescription, ScratchBuffer,
    },
    shader_binding_table::RayTracingShaderBindingTables,
};
use crate::{core::command::CommandBufferRecorder, device, features::Feature};
use log::debug;

pub struct RayTracingFeature {}

impl RayTracingFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for RayTracingFeature {
    fn device_extensions(&self) -> Vec<device::Ext> {
        vec![
            device::Ext::AccelerationStructure,
            device::Ext::DeferredHostOperations,
            device::Ext::RayTracingPipeline,
        ]
    }
}

impl CommandBufferRecorder<'_> {
    pub fn trace_rays(
        &self,
        binding_tables: &RayTracingShaderBindingTables,
        size: (u32, u32, u32),
    ) {
        // info!("binding tables: {:?}", binding_tables);
        unsafe {
            self.device().ext().ray_tracing_pipeline().cmd_trace_rays(
                self.buffer().raw(),
                binding_tables.raygen.raw(),
                binding_tables.miss.raw(),
                binding_tables.hit.raw(),
                binding_tables.callable.raw(),
                size.0,
                size.1,
                size.2,
            )
        }
    }

    pub fn build_acceleration_structure(
        &self,
        description: &AccelerationStructureDescription,
        destination: &AccelerationStructure,
        scratch: &ScratchBuffer,
    ) {
        let description = description.bind_for_build(destination, scratch);
        debug!("geo: {:?}", unsafe {
            *description.geometry_info().p_geometries
        });
        debug!("ranges: {:?}", description.ranges());
        unsafe {
            self.device()
                .ext()
                .acceleration_structure()
                .cmd_build_acceleration_structures(
                    self.buffer().raw(),
                    &[description.geometry_info()],
                    &[description.ranges()],
                );
        }
    }
}
