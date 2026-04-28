use glam::Vec4Swizzles;

use crate::core::camera::CameraUniform;
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

    // pub fn calc_view_matrix(&self) -> Mat4 {
    //     Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    // }

    // pub fn calc_projection_matrix(&self) -> Mat4 {
    //     Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    // }

    // /// Матрица для скайбокса: (Proj * ViewRotationOnly).inverse()
    // pub fn calc_inv_skybox_view_proj(&self) -> Mat4 {
    //     let view = self.calc_view_matrix();
    //     let proj = self.calc_projection_matrix();

    //     let rotation_only = Mat4::from_mat3(view.mat4_to_mat3());
    //     (proj * rotation_only).inverse()
    // }

    // pub fn calc_matrix(&self) -> Mat4 {
    //     let view = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
    //     let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
    //     proj * view
    // }

    pub fn get_uniform(&self) -> CameraUniform {
        let view = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far);

        let view_proj = proj * view;

        let rotation_only =
            glam::Mat3::from_cols(view.x_axis.xyz(), view.y_axis.xyz(), view.z_axis.xyz());
        let inv_skybox_view_proj = (proj * Mat4::from_mat3(rotation_only)).inverse();

        CameraUniform {
            view_proj,
            inv_skybox_view_proj,
            position: self.position.extend(1.0),
        }
    }
}
