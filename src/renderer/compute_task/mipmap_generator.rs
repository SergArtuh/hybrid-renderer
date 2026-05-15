use std::sync::Arc;

use crate::core::{
    compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
    render_context::RenderContext,
    texture::Texture,
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::MipmapGenerator;

pub struct TaskDescriptor {
    pub source_texture: Arc<Texture>,
    pub output_texture: Arc<Texture>,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        "mipmap_generator.wgsl"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    // Читаем как Cube (или D2Array, если по слоям)
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    // Пишем в массив из 6 слоев
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ]
    }

    fn create_instance(
        render_context: &RenderContext,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let current_mip_view =
            desc.source_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Source Cube View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                });

        let target_mip_view =
            desc.output_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Scratch Storage View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&current_mip_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&target_mip_view),
                    },
                ],
                label: Some("Clear Cubemap Bind Group"),
            });

        let mip_width = desc.output_texture.width;
        let mip_height = desc.output_texture.height;
        let workgroup_size = 8;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
            dispatch_size: (
                (mip_width + workgroup_size - 1) / workgroup_size,
                (mip_height + workgroup_size - 1) / workgroup_size,
                6,
            ),
        })
    }

    type Descriptor = TaskDescriptor;
}
