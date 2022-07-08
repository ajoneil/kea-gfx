use kea_gpu_shaderlib::Aabb;
use spirv_std::glam::{vec3, Vec3};

// Needed for .sqrt()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

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

    pub fn intersect_ray(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32> {
        let oc = ray_origin - self.position();
        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * oc.dot(ray_direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = b * b - (4.0 * a * c);

        if discriminant >= 0.0 {
            Some((-b - discriminant.sqrt()) / (2.0 * a))
        } else {
            None
        }
    }
}
