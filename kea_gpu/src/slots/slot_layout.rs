use crate::descriptors::DescriptorSetLayoutBinding;
use ash::vk;
use kea_gpu_shaderlib::slots::{ShaderStage, Slot, SlotType};

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

                let stage_flags = match slot.stage {
                    ShaderStage::RayGen => vk::ShaderStageFlags::RAYGEN_KHR,
                    ShaderStage::Intersection => vk::ShaderStageFlags::INTERSECTION_KHR,
                };

                DescriptorSetLayoutBinding::new(index as _, descriptor_type, 1, stage_flags)
            })
            .collect()
    }
}
