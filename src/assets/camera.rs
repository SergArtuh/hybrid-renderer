use crate::core::math::Mat4;
use crate::core::math::Vec3;

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            target: Vec3::ZERO,
            aspect: 1.0,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
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

    pub fn with_near(mut self, near: f32) -> Self {
        self.near = near;
        self
    }

    pub fn with_far(mut self, far: f32) -> Self {
        self.far = far;
        self
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        proj * view
    }
}
