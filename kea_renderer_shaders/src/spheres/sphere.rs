use kea_gpu_shaderlib::{Aabb, Ray};
use spirv_std::glam::{vec3, Vec3, Vec3A};

// Needed for .sqrt()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::materials::Material;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    position: Vec3A,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32, material: Material) -> Self {
        Self {
            position: Vec3A::from(position),
            radius,
            material,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb {
            min: Vec3::from(self.position) - Vec3::splat(self.radius),
            max: Vec3::from(self.position) + Vec3::splat(self.radius),
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::from(self.position)
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn intersect_ray(&self, ray: Ray) -> Option<f32> {
        // A sphere's implicit formula is:
        // `||x - c||^2 = r^2`
        //  where c is the centre and r is the radius
        // We want to find the distance travelled along the ray that this formula
        // holds true. We can use the quadaratic formula to calculate the zero - two
        // solutions (doesn't intersect, touches at a single point, or intersects on
        // entry and exit).
        let oc = ray.origin - self.center();
        // A vector's dot product with itself is the length squared, so using this
        // directly is a performance optimisation.
        let a = ray.direction.length_squared();
        let c = oc.length_squared() - (self.radius * self.radius);
        // h = half of b. By doing this we can cancel out some constants.
        let h = oc.dot(ray.direction);
        let discriminant = h * h - (a * c);

        if discriminant < 0.0 {
            return None;
        }

        // We've calculated a line-sphere intersection, but rays only extend in
        // one direction from a point, so we need to discard values behind the
        // ray origin (ie with a negative distance).
        let distance = (-h - discriminant.sqrt()) / a;
        if distance > 0.0 {
            return Some(distance);
        }

        let distance = (-h + discriminant.sqrt()) / a;
        if distance > 0.0 {
            Some(distance)
        } else {
            None
        }
    }

    pub fn normal(&self, ray: Ray) -> Vec3 {
        if let Some(distance) = self.intersect_ray(ray) {
            (ray.at(distance) - self.center()).normalize()
        } else {
            // Garbage in, garbage out
            vec3(0.0, 0.0, 0.0)
        }
    }

    pub fn material(&self) -> Material {
        self.material
    }
}
