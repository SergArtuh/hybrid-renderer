use std::sync::Arc;

use crate::core::{
    compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
    render_context::RenderContext,
    texture::Texture,
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::ClearCubemap;

pub struct TaskDescriptor {
    pub cubemap: Arc<Texture>,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        "clear_cubemap.wgsl"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba16Float, // TODO: make it configurable
                view_dimension: wgpu::TextureViewDimension::D2Array,
            },
            count: None,
        }]
    }

    fn create_instance(
        render_context: &RenderContext,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let cubemap_view = desc
            .cubemap
            .array_view
            .as_ref()
            .expect("Cubemap array view should be present");

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                }],
                label: Some("Clear Cubemap Bind Group"),
            });

        let (width, height) = (desc.cubemap.width, desc.cubemap.height);
        let workgroup_size = 16;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
            dispatch_size: (
                (width + workgroup_size - 1) / workgroup_size,
                (height + workgroup_size - 1) / workgroup_size,
                6,
            ),
        })
    }

    type Descriptor = TaskDescriptor;
}
