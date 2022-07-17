use crate::materials::Material;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Mesh {
    // pub vertices_address: u64,
    // pub indices_address: u64,
    pub vertices_offset: u32,
    pub indices_offset: u32,
    pub material: Material,
}
