use spirv_std::glam::Vec3;

#[repr(C)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}
