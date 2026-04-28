use crate::core::math::{Mat4, Vec4};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: Mat4,
    pub inv_skybox_view_proj: Mat4,
    pub position: Vec4,
}
