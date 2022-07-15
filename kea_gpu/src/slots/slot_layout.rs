use crate::descriptors::DescriptorSetLayoutBinding;
use ash::vk;
use kea_gpu_shaderlib::slots::{Slot, SlotType};

pub struct SlotLayout<SlotId> {
    slots: Vec<(SlotId, Slot)>,
}

impl<SlotId> SlotLayout<SlotId> {
    pub fn new(slots: Vec<(SlotId, Slot)>) -> Self {
        Self { slots }
    }

    pub fn bindings(&self) -> Vec<DescriptorSetLayoutBinding> {
        self.slots
            .iter()
            .enumerate()
            .map(|(index, (_, slot))| {
                let descriptor_type = match slot.slot_type {
                    SlotType::AccelerationStructure => {
                        vk::DescriptorType::ACCELERATION_STRUCTURE_KHR
                    }
                    SlotType::Image => vk::DescriptorType::STORAGE_IMAGE,
                    SlotType::Buffer(_) => vk::DescriptorType::STORAGE_BUFFER,
                };

                let mut stage_flags = vk::ShaderStageFlags::empty();
                if slot.stages.raygen {
                    stage_flags |= vk::ShaderStageFlags::RAYGEN_KHR
                }
                if slot.stages.intersection {
                    stage_flags |= vk::ShaderStageFlags::INTERSECTION_KHR
                }
                if slot.stages.closest_hit {
                    stage_flags |= vk::ShaderStageFlags::CLOSEST_HIT_KHR
                }

                DescriptorSetLayoutBinding::new(index as _, descriptor_type, 1, stage_flags)
            })
            .collect()
    }

    pub fn slots(&self) -> &[(SlotId, Slot)] {
        &self.slots
    }
}
