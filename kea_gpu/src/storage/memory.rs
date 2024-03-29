use ash::vk;
use gpu_allocator::{
    vulkan::{AllocationCreateDesc, AllocationScheme},
    MemoryLocation,
};
use num_traits::{PrimInt, Unsigned};
use std::{mem::ManuallyDrop, os::raw::c_void, sync::Arc};

use crate::device::Device;

pub fn align<T: PrimInt + Unsigned + From<u8>>(size_or_address: T, alignment: T) -> T {
    (size_or_address + (alignment - <T as From<u8>>::from(1)))
        & !(alignment - <T as From<u8>>::from(1))
}

pub struct Allocation {
    device: Arc<Device>,
    allocation: ManuallyDrop<gpu_allocator::vulkan::Allocation>,
}

impl Allocation {
    pub fn new(
        device: Arc<Device>,
        name: String,
        location: MemoryLocation,
        requirements: vk::MemoryRequirements,
    ) -> Self {
        let allocation = device
            .allocator()
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: &name,
                requirements,
                location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        Self {
            device,
            allocation: ManuallyDrop::new(allocation),
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

    pub unsafe fn data_ptr(&mut self) -> *mut c_void {
        self.allocation.mapped_ptr().unwrap().as_ptr()
    }
}

impl Drop for Allocation {
    fn drop(&mut self) {
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
