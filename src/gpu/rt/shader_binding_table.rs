use ash::vk;

pub struct RayTracingShaderBindingTables {
    pub raygen: ShaderBindingTable,
    pub miss: ShaderBindingTable,
    pub hit: ShaderBindingTable,
    pub callable: ShaderBindingTable,
}

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
