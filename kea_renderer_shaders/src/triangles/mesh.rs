use crate::materials::Material;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Mesh {
    pub material: Material,
}
