use crate::core::render_context::RenderContext;
use crate::stage::frame_data::RenderItem;

#[derive(Debug)]
pub struct ModelManager {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) stride: wgpu::BufferAddress,
}

impl ModelManager {
    const MAX_MODELS: u64 = 100;

    pub fn new(render_context: &RenderContext, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let matrix_size = std::mem::size_of::<glam::Mat4>() as wgpu::BufferAddress;
        let alignment = render_context
            .device
            .limits()
            .min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let dynamic_offset_step = (matrix_size + alignment - 1) & !(alignment - 1);

        let buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Global Model Matrix Buffer"),
                size: dynamic_offset_step * Self::MAX_MODELS,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(matrix_size),
                    }),
                }],
                label: Some("camera_bind_group"),
            });

        let stride = Self::calculate_stride(render_context);

        Self {
            buffer,
            bind_group,
            stride,
        }
    }

    pub fn update_buffer(&self, render_context: &RenderContext, items: &[RenderItem]) {
        let matrix_size = std::mem::size_of::<glam::Mat4>();

        let mut data = Vec::with_capacity(items.len() * self.stride as usize);

        for item in items {
            let matrix = item.world_matrix();
            let bytes = bytemuck::cast_slice(matrix.as_ref());
            data.extend_from_slice(bytes);
            let padding = self.stride as usize - matrix_size;
            data.extend(std::iter::repeat(0).take(padding));
        }

        render_context.queue.write_buffer(&self.buffer, 0, &data);
    }

    fn calculate_stride(render_context: &RenderContext) -> wgpu::BufferAddress {
        let matrix_size = std::mem::size_of::<glam::Mat4>();
        let alignment = render_context
            .device
            .limits()
            .min_uniform_buffer_offset_alignment as usize;
        let stride: usize = (matrix_size + alignment - 1) & !(alignment - 1);
        stride as wgpu::BufferAddress
    }
}
