use crate::core::math::{Mat4, Vec4};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: Mat4,
    pub inv_skybox_view_proj: Mat4,
    pub position: Vec4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkydomeUniform {
    // [radius, dome_factor, 0.0, 0.0]
    pub data: [f32; 4],
}

impl SkydomeUniform {
    pub fn new(radius: f32, dome_factor: f32) -> Self {
        Self {
            data: [radius, dome_factor, 0.0, 0.0],
        }
    }
}
