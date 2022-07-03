use super::{Slot, SlotType};
use core::any::TypeId;

pub fn slot_shader_type(slot: &Slot) -> TypeId {
    match slot.slot_type {
        SlotType::AccelerationStructure => {
            TypeId::of::<&spirv_std::ray_tracing::AccelerationStructure>()
        }
        SlotType::Image => {
            TypeId::of::<&mut spirv_std::Image!(2D, format=rgba32f, sampled=false)>()
        }
        SlotType::Buffer(type_id) => type_id,
    }
}
