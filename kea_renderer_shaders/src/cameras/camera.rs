use spirv_std::glam::{vec3, Vec2, Vec3};

pub struct Camera {
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }

    pub fn ray_target(&self, uv: Vec2) -> Vec3 {
        let direction = uv * 2.0 - 1.0;
        let target = vec3(direction.x * self.aspect_ratio, direction.y, -1.0);
        target.normalize()
    }
}
