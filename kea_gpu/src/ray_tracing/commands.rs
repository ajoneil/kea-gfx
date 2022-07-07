use std::slice;

use ash::vk;

use crate::commands::CommandBufferRecorder;

use super::RayTracingShaderBindingTables;

impl CommandBufferRecorder<'_> {
    pub fn trace_rays(
        &self,
        binding_tables: &RayTracingShaderBindingTables,
        size: (u32, u32, u32),
    ) {
        // log::info!("binding tables: {:?}", binding_tables);
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
        geometry_info: &vk::AccelerationStructureBuildGeometryInfoKHR,
        range: &vk::AccelerationStructureBuildRangeInfoKHR,
    ) {
        unsafe {
            self.device()
                .ext()
                .acceleration_structure()
                .cmd_build_acceleration_structures(
                    self.buffer().raw(),
                    slice::from_ref(geometry_info),
                    slice::from_ref(&slice::from_ref(range)),
                );
        }
    }
}
