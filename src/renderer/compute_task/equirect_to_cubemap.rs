use std::sync::Arc;

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
        texture::Texture,
        texture_builder::TextureBuilder,
    },
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};
pub struct TaskDescriptor {
    pub input_texture: Arc<Texture>,
    pub output_cubemap: Arc<Texture>,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
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
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                    format: wgpu::TextureFormat::Rgba16Float, // TODO: make it configurable
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let cubemap_view = desc
            .output_cubemap
            .array_view
            .as_ref()
            .expect("Cubemap array view should be present");

        let bind_group =
            render_env
                .render_context
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
                            resource: wgpu::BindingResource::TextureView(&cubemap_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(
                                &render_env.render_resources.common_sampler,
                            ),
                        },
                    ],
                    label: Some("Equirect to Cubemap Bind Group"),
                });

        let (width, height) = (desc.output_cubemap.width, desc.output_cubemap.height);
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

pub struct Provider<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

impl<'a> Provider<'a> {
    pub fn new(render_env: &'a RenderingEnvironment<'a>) -> Self {
        Self { render_env }
    }

    pub fn process(&self, source_texture: &Arc<Texture>) -> Result<Arc<Texture>, anyhow::Error> {
        let result_texture = Arc::new(
            TextureBuilder::new(
                &self.render_env.render_context.device,
                &self.render_env.render_context.queue,
            )
            .with_label("equirect_cubemap")
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_size(1024, 1024)
            .as_cubemap()
            .with_usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .build(),
        );
        let compute_task_factory = ComputeTaskFactory::new(self.render_env);
        let equirect_to_cubemap_task = compute_task_factory.create_task::<Task>(TaskDescriptor {
            input_texture: source_texture.clone(),
            output_cubemap: result_texture.clone(),
        });

        ComputeTaskFactory::new(&self.render_env)
            .create_executor()
            .record(&equirect_to_cubemap_task)
            .execute()
            .wait();

        Ok(result_texture)
    }
}
