#[derive(PartialEq)]
#[repr(C)]
pub enum HitType {
    Hit,
    Miss,
}

#[repr(C)]
pub struct RayPayload {
    pub hit_type: HitType,
}
