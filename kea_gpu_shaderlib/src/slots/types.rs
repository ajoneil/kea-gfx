use core::any::TypeId;

#[derive(Clone)]
pub enum SlotType {
    AccelerationStructure,
    Image,
    Buffer(TypeId),
}
