use crate::core::camera::CameraUniform;
use crate::core::render_context::RenderContext;

#[derive(Debug)]
pub struct CameraManager {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl CameraManager {
    pub fn new(render_context: &RenderContext, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: std::mem::size_of::<CameraUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }],
                label: Some("camera_bind_group"),
            });

        Self { buffer, bind_group }
    }

    pub fn update_buffer(&self, render_context: &RenderContext, camera_uniform: &CameraUniform) {
        render_context.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[*camera_uniform]),
        );
    }
}
