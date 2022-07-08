use super::types::SlotType;

#[derive(Clone, Default)]
pub struct ShaderStages {
    pub raygen: bool,
    pub intersection: bool,
    pub closest_hit: bool,
}

#[derive(Clone)]
pub struct Slot {
    pub slot_type: SlotType,
    pub stages: ShaderStages,
}

impl Slot {
    pub const fn new(slot_type: SlotType, stages: ShaderStages) -> Self {
        Self { slot_type, stages }
    }
}
