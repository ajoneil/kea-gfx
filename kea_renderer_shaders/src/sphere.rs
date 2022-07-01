use kea_gpu_shaderlib::Aabb;
use spirv_std::glam::{vec3, Vec3};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[repr(C)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn aabb(&self) -> Aabb {
        let Sphere { position, radius } = self;

        Aabb {
            min: vec3(
                position.x - radius,
                position.y - radius,
                position.z - radius,
            ),
            max: vec3(
                position.x + radius,
                position.y + radius,
                position.z + radius,
            ),
        }
    }
}
