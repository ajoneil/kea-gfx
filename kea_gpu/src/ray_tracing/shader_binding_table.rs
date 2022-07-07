use crate::{
    device::Device,
    pipelines::Pipeline,
    shaders::{PipelineShaders, ShaderGroups},
    storage::{buffers::Buffer, memory},
};
use ash::vk;
use kea_gpu_shaderlib::shaders::ShaderGroup;
use std::{iter, sync::Arc};

pub struct RayTracingShaderBindingTables {
    pub raygen: ShaderBindingTable,
    pub miss: ShaderBindingTable,
    pub hit: ShaderBindingTable,
    pub callable: ShaderBindingTable,
    _buffer: Buffer,
}

impl RayTracingShaderBindingTables {
    pub fn new<ShaderGroupId>(
        device: &Arc<Device>,
        shader_groups: &ShaderGroups<ShaderGroupId>,
        shaders: &PipelineShaders,
        pipeline: &Pipeline,
    ) -> Self {
        let vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
            shader_group_handle_size,
            shader_group_handle_alignment,
            shader_group_base_alignment,
            ..
        } = device.physical_device().ray_tracing_pipeline_properties();

        let group_handles = unsafe {
            device
                .ext()
                .ray_tracing_pipeline()
                .get_ray_tracing_shader_group_handles(
                    pipeline.raw(),
                    0,
                    shaders.groups.len() as _,
                    shaders.groups.len() as usize * shader_group_handle_size as usize,
                )
        }
        .unwrap();

        let mut raygen: Vec<u8> = vec![];
        let mut miss: Vec<u8> = vec![];
        let mut hit: Vec<u8> = vec![];

        let shader_group_handle_aligned_size =
            memory::align(shader_group_handle_size, shader_group_handle_alignment);
        for ((_, group), handle) in shader_groups
            .groups()
            .iter()
            .zip(group_handles.chunks(shader_group_handle_size as _))
        {
            match group {
                ShaderGroup::RayGeneration(_) => {
                    raygen.extend_from_slice(handle);
                    raygen.extend(iter::repeat(0).take(
                        shader_group_handle_aligned_size as usize
                            - shader_group_handle_size as usize,
                    ));
                }
                ShaderGroup::Miss(_) => {
                    miss.extend_from_slice(handle);
                    miss.extend(iter::repeat(0).take(
                        shader_group_handle_aligned_size as usize
                            - shader_group_handle_size as usize,
                    ));
                }
                ShaderGroup::TriangleHit { .. } => {
                    hit.extend_from_slice(handle);
                    hit.extend(iter::repeat(0).take(
                        shader_group_handle_aligned_size as usize
                            - shader_group_handle_size as usize,
                    ));
                }
                ShaderGroup::ProceduralHit { .. } => {
                    hit.extend_from_slice(handle);
                    hit.extend(iter::repeat(0).take(
                        shader_group_handle_aligned_size as usize
                            - shader_group_handle_size as usize,
                    ));
                }
            }
        }

        let mut binding_table_data: Vec<u8> = vec![];
        for group in [&mut raygen, &mut miss, &mut hit] {
            let aligned_size = memory::align(group.len(), shader_group_base_alignment as _);
            group.extend(iter::repeat(0).take(aligned_size - group.len()));
            binding_table_data.extend(group.iter());
        }

        let buffer = Buffer::new_from_data(
            device.clone(),
            &binding_table_data,
            vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR,
            "rt shader binding table".to_string(),
            gpu_allocator::MemoryLocation::GpuOnly,
            Some(shader_group_base_alignment as _),
        );
        let buffer_address = buffer.device_address();

        Self {
            raygen: ShaderBindingTable::new(buffer_address, raygen.len() as _, raygen.len() as _),

            miss: ShaderBindingTable::new(
                buffer_address + raygen.len() as u64,
                miss.len() as _,
                shader_group_handle_aligned_size as _,
            ),

            hit: ShaderBindingTable::new(
                buffer_address + raygen.len() as u64 + miss.len() as u64,
                hit.len() as _,
                shader_group_handle_aligned_size as _,
            ),

            callable: ShaderBindingTable::empty(),
            _buffer: buffer,
        }
    }
}

#[derive(Debug)]
pub struct ShaderBindingTable {
    raw: vk::StridedDeviceAddressRegionKHR,
}

impl ShaderBindingTable {
    pub fn new(device_address: u64, size: u64, stride: u64) -> ShaderBindingTable {
        let raw = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(device_address)
            .size(size)
            .stride(stride)
            .build();

        ShaderBindingTable { raw }
    }

    pub fn empty() -> ShaderBindingTable {
        let raw = vk::StridedDeviceAddressRegionKHR::builder().build();

        ShaderBindingTable { raw }
    }

    pub unsafe fn raw(&self) -> &vk::StridedDeviceAddressRegionKHR {
        &self.raw
    }
}
