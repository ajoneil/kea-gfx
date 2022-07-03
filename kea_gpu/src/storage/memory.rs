use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, MemoryLocation};
use num_traits::{PrimInt, Unsigned};
use std::{mem::ManuallyDrop, sync::Arc};

use crate::device::Device;

pub fn align<T: PrimInt + Unsigned + From<u8>>(size_or_address: T, alignment: T) -> T {
    (size_or_address + (alignment - <T as From<u8>>::from(1)))
        & !(alignment - <T as From<u8>>::from(1))
}

pub struct Allocation {
    device: Arc<Device>,
    allocation: ManuallyDrop<gpu_allocator::vulkan::Allocation>,
    name: String,
}

impl Allocation {
    pub fn new(
        device: Arc<Device>,
        name: String,
        location: MemoryLocation,
        requirements: vk::MemoryRequirements,
    ) -> Self {
        log::debug!("Allocating {:?}", name);

        let allocation = device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: &name,
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        Self {
            device,
            allocation: ManuallyDrop::new(allocation),
            name,
        }
    }

    pub unsafe fn memory(&self) -> vk::DeviceMemory {
        self.allocation.memory()
    }

    pub unsafe fn offset(&self) -> u64 {
        self.allocation.offset()
    }

    pub fn size(&self) -> u64 {
        self.allocation.size()
    }

    pub unsafe fn mapped_slice_mut(&mut self) -> &mut [u8] {
        self.allocation.mapped_slice_mut().unwrap()
    }
}

impl Drop for Allocation {
    fn drop(&mut self) {
        log::debug!("Freeing {:?}", self.name);

        unsafe {
            self.device
                .allocator()
                .lock()
                .unwrap()
                .free(ManuallyDrop::take(&mut self.allocation))
                .unwrap();
        }
    }
}
