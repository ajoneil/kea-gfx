use super::types::SlotType;

#[derive(Clone)]
pub enum ShaderStage {
    RayGen,
    Intersection,
}

#[derive(Clone)]
pub struct Slot {
    pub slot_type: SlotType,
    pub stage: ShaderStage,
}

impl Slot {
    pub const fn new(slot_type: SlotType, stage: ShaderStage) -> Self {
        Self { slot_type, stage }
    }
}
