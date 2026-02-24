use crate::core::math::Mat4;
use crate::core::math::Vec3;

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub aspect: f32,
    pub fov: f32,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            target: Vec3::ZERO,
            aspect: 1.0,
            fov: 45.0,
        }
    }

    pub fn with_target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    pub fn with_aspect(mut self, aspect: f32) -> Self {
        self.aspect = aspect;
        self
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, 0.1, 100.0);
        proj * view
    }
}
