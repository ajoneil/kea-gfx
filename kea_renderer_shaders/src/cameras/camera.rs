use kea_gpu_shaderlib::Ray;
use spirv_std::glam::{vec3, Vec3};

// Needed for .tan()
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub struct CameraParameters {
    pub aspect_ratio: f32,
    pub vertical_field_of_view_radians: f32,
    pub position: Vec3,
    pub target_position: Vec3,
    pub up_direction: Vec3,
}

impl Default for CameraParameters {
    fn default() -> Self {
        Self {
            aspect_ratio: 4.0 / 3.0,
            vertical_field_of_view_radians: 90.0_f32.to_radians(),
            position: vec3(0.0, 0.0, 0.0),
            target_position: vec3(0.0, 0.0, -1.0),
            up_direction: vec3(0.0, 1.0, 0.0),
        }
    }
}

pub struct Camera {
    position: Vec3,
    lower_left: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new(params: CameraParameters) -> Self {
        let height = 2.0 * (params.vertical_field_of_view_radians / 2.0).tan();
        let width = height * params.aspect_ratio;

        // https://raytracing.github.io/books/RayTracingInOneWeekend.html#positionablecamera
        let w = (params.position - params.target_position).normalize();
        let u = params.up_direction.cross(w).normalize();
        let v = w.cross(u);

        let horizontal = width * u;
        let vertical = height * v;

        Self {
            position: params.position,
            horizontal,
            vertical,
            lower_left: params.position - horizontal / 2.0 - vertical / 2.0 - w,
        }
    }

    pub fn ray(&self, s: f32, t: f32) -> Ray {
        Ray {
            origin: self.position,
            direction: self.lower_left + s * self.horizontal + t * self.vertical - self.position,
        }
    }
}
