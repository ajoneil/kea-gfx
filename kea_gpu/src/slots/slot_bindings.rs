use crate::{
    descriptors::{DescriptorPool, DescriptorSet},
    device::Device,
    ray_tracing::{scenes::AccelerationStructure, RayTracingPipeline},
    storage::{buffers::Buffer, images::ImageView},
};
use ash::vk;
use kea_gpu_shaderlib::slots::SlotType;
use std::{collections::HashMap, hash::Hash, slice, sync::Arc};

pub struct SlotBindings<SlotId> {
    descriptor_set: DescriptorSet,
    buffers: HashMap<SlotId, Arc<Buffer>>,
    acceleration_structures: HashMap<SlotId, Arc<AccelerationStructure>>,
    images: HashMap<SlotId, Arc<ImageView>>,
}

impl<SlotId: Into<u32> + Hash + Eq + Copy> SlotBindings<SlotId> {
    pub fn new(device: Arc<Device>, pipeline: &RayTracingPipeline<SlotId>) -> Self {
        let pool_sizes: Vec<_> = pipeline
            .slot_layout()
            .slots()
            .iter()
            .map(|(_, slot)| vk::DescriptorPoolSize {
                ty: match slot.slot_type {
                    SlotType::AccelerationStructure => {
                        vk::DescriptorType::ACCELERATION_STRUCTURE_KHR
                    }
                    SlotType::Buffer(_) => vk::DescriptorType::STORAGE_BUFFER,
                    SlotType::Image => vk::DescriptorType::STORAGE_IMAGE,
                },
                descriptor_count: 1,
            })
            .collect();

        let descriptor_pool = DescriptorPool::new(device, 1, &pool_sizes);
        let descriptor_sets = descriptor_pool
            .allocate_descriptor_sets(slice::from_ref(pipeline.layout().descriptor_set_layout()));
        let descriptor_set = descriptor_sets.into_iter().nth(0).unwrap();

        Self {
            descriptor_set,
            buffers: HashMap::new(),
            acceleration_structures: HashMap::new(),
            images: HashMap::new(),
        }
    }

    pub fn bind_buffer(&mut self, slot_id: SlotId, buffer: Arc<Buffer>) {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: unsafe { buffer.buffer().raw() },
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_set(unsafe { self.descriptor_set.raw() })
            .dst_binding(slot_id.into())
            .buffer_info(slice::from_ref(&buffer_info));

        unsafe {
            self.device()
                .raw()
                .update_descriptor_sets(slice::from_ref(&write_set), &[])
        };

        self.buffers.insert(slot_id, buffer);
    }

    pub fn bind_acceleration_structure(
        &mut self,
        slot_id: SlotId,
        acceleration_structure: Arc<AccelerationStructure>,
    ) {
        let accel_raw = unsafe { acceleration_structure.raw() };
        let accel_slice = std::slice::from_ref(&accel_raw);
        let mut write_set_as = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(accel_slice);
        let mut write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .dst_set(unsafe { self.descriptor_set.raw() })
            .dst_binding(slot_id.into())
            .push_next(&mut write_set_as)
            .build();
        write_set.descriptor_count = 1;

        unsafe {
            self.device()
                .raw()
                .update_descriptor_sets(slice::from_ref(&write_set), &[])
        };

        self.acceleration_structures
            .insert(slot_id, acceleration_structure);
    }

    pub fn bind_image(&mut self, slot_id: SlotId, image: Arc<ImageView>) {
        let desc_img_info = vk::DescriptorImageInfo::builder()
            .image_view(unsafe { image.raw() })
            .image_layout(vk::ImageLayout::GENERAL);

        let write_set = vk::WriteDescriptorSet::builder()
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .dst_set(unsafe { self.descriptor_set.raw() })
            .dst_binding(slot_id.into())
            .image_info(slice::from_ref(&desc_img_info));

        unsafe {
            self.device()
                .raw()
                .update_descriptor_sets(slice::from_ref(&write_set), &[])
        };

        self.images.insert(slot_id, image);
    }

    pub fn device(&self) -> &Arc<Device> {
        self.descriptor_set.device()
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.descriptor_set
    }
}
