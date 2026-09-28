use std::sync::Arc;

pub struct FrameTarget {
    pub surface_texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub normal_buffer_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}
