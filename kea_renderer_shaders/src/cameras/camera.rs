use spirv_std::glam::{vec3, Vec3};

// Needed for .tan()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub struct Camera {
    height: f32,
    width: f32,
    focal_length: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32, vertical_field_of_view_radians: f32, focal_length: f32) -> Self {
        let height = 2.0 * (vertical_field_of_view_radians / 2.0).tan();
        let width = height * aspect_ratio;

        Self {
            height,
            width,
            focal_length,
        }
    }

    pub fn ray_target(&self, u: f32, v: f32) -> Vec3 {
        vec3(
            -self.width / 2.0 + u * self.width,
            -self.height / 2.0 + v * self.height,
            -self.focal_length,
        )
        .normalize()
    }
}
