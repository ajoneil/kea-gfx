use kea_gpu_shaderlib::{Aabb, Ray};
use spirv_std::glam::{vec3, Vec3};

use crate::materials::Material;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Boxo {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    dim_x: f32,
    dim_y: f32,
    dim_z: f32,
    material: Material,
}

impl Boxo {
    pub fn new(position: Vec3, dimensions: Vec3, material: Material) -> Self {
        Self {
            pos_x: position.x,
            pos_y: position.y,
            pos_z: position.z,
            dim_x: dimensions.x,
            dim_y: dimensions.y,
            dim_z: dimensions.z,
            material,
        }
    }

    pub fn position(&self) -> Vec3 {
        vec3(self.pos_x, self.pos_y, self.pos_z)
    }

    pub fn dimensions(&self) -> Vec3 {
        vec3(self.dim_x, self.dim_y, self.dim_z)
    }

    pub fn aabb(&self) -> Aabb {
        let halfed_dimensions = self.dimensions() / 2.0;
        Aabb {
            min: self.position() - halfed_dimensions,
            max: self.position() + halfed_dimensions,
        }
    }

    pub fn intersect_ray(&self, ray: Ray) -> Option<f32> {
        let bounds = self.aabb();

        let direction_inverted = 1.0 / ray.direction;
        let tbot = direction_inverted * (bounds.min - ray.origin);
        let ttop = direction_inverted * (bounds.max - ray.origin);
        let tmin = ttop.min(tbot);
        let tmax = ttop.max(tbot);
        let t0 = tmin.x.max(tmin.y).max(tmin.z);
        let t1 = tmax.x.min(tmax.y).min(tmax.z);

        let min = t0.min(t1);
        if min > 0.0 {
            Some(min)
        } else {
            let max = t0.max(t1);

            if max > 0.0 {
                Some(max)
            } else {
                None
            }
        }
    }

    pub fn normal(&self, ray: Ray) -> Vec3 {
        let distance = self.intersect_ray(ray).unwrap();

        let normal = (ray.at(distance) - self.position()).normalize();
        let normal_absolute = normal.abs();
        let sign = normal.signum();

        if normal_absolute.x > normal_absolute.y && normal_absolute.x > normal_absolute.z {
            vec3(sign.x, 0.0, 0.0)
        } else if normal_absolute.y > normal_absolute.z {
            vec3(0.0, sign.y, 0.0)
        } else {
            vec3(0.0, 0.0, sign.z)
        }
    }

    pub fn material(&self) -> Material {
        self.material
    }
}
