use std::sync::Arc;

use crate::core::{
    compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
    render_context::RenderContext,
    texture::Texture,
};
pub struct EquirectToCubemapTaskDescriptor {
    pub input_texture: Arc<Texture>,
    pub output_cubemap: Arc<Texture>,
}
pub struct EquirectToCubemapTask {
    //pub input_texture: Arc<Texture>,
    //pub output_cubemap: Arc<Texture>,
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for EquirectToCubemapTask {
    const TYPE: ComputeTaskType = ComputeTaskType::EquirectToCubemap;

    fn get_shader_path() -> &'static str {
        "equirect_to_cubemap.wgsl"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float, // TODO: make it configurable
                    view_dimension: wgpu::TextureViewDimension::D2,
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
        // let task_definition = EquirectToCubemapTaskDefinition::default();
        // let task = task_definition.create_instance(render_context, desc, layout);
        // Ok(task)

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&desc.input_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&desc.output_cubemap.view),
                    },
                ],
                label: Some("Equirect to Cubemap Bind Group"),
            });

        let (width, height) = (desc.input_texture.width, desc.input_texture.height);
        let workgroup_size = 16;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
            dispatch_size: (
                (width + workgroup_size - 1) / workgroup_size,
                (height + workgroup_size - 1) / workgroup_size,
                1,
            ),
        })

        // Ok(ComputeTaskInstance {
        //     task_type: ComputeTaskType::EquirectToCubemap,
        //     bind_group,
        //     dispatch_size: (1, 1, 1),
        // })
    }

    type Descriptor = EquirectToCubemapTaskDescriptor;
}

/*
#[derive(Default, Clone)]
struct EquirectToCubemapTaskDefinition;

impl EquirectToCubemapTaskDefinition {
    pub fn create_instance(
        &self,
        render_context: &RenderContext,
        desc: EquirectToCubemapTaskDescriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> ComputeTaskInstance {
        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&desc.input_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &render_context.common_nearest_sampler,
                        ),
                    },
                ],
                label: Some("Sprite material bind group"),
            });

        ComputeTaskInstance {
            task_type: ComputeTaskType::EquirectToCubemap,
            bind_group,
            dispatch_size: (1, 1, 1),
        }
        //     let pipeline = pipeline_manager.get_compute_pipeline(T::TYPE);

        //     // Создаем BindGroup один раз при инициализации
        //     let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some(&format!("{:?} BindGroup", T::TYPE)),
        //         layout: &pipeline.get_bind_group_layout(0),
        //         entries,
        //     });

        //     ComputeTaskInstance {
        //         task_type: T::TYPE,
        //         bind_group,
        //         dispatch_size: (1, 1, 1),
        //     }
        //
    }

    // pub fn create_pipeline(&self, render_context: &RenderContext) -> wgpu::ComputePipeline {
    //     let shader_module = render_context
    //         .shader_cache
    //         .get_or_create_module("equirect_to_cubemap.wgsl");
    // }
}
    */

// pub struct ComputePipelineDefinition {
//     pub task_type: ComputeTaskType,
//     pub shader_path: std::path::PathBuf,
//     pub entry_point: String,
// }
