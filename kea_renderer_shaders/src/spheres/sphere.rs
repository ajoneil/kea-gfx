use kea_gpu_shaderlib::Aabb;
use spirv_std::glam::{vec3, Vec3};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Copy, Clone)]
#[repr(align(32))]
pub struct Sphere {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: position.z,
            radius,
        }
    }

    pub fn aabb(&self) -> Aabb {
        let Sphere { x, y, z, radius } = self;

        Aabb {
            min: vec3(x - radius, y - radius, z - radius),
            max: vec3(x + radius, y + radius, z + radius),
        }
    }

    pub fn position(&self) -> Vec3 {
        vec3(self.x, self.y, self.z)
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }
}
