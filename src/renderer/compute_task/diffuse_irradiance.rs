use std::sync::Arc;

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
        render_context::RenderContext,
        texture::Texture,
        texture_builder::TextureBuilder,
    },
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::DiffuseIrradiance;
const SHADER_PATH: &str = "diffuse_irradiance.wgsl";

pub struct TaskDescriptor {
    pub input_cubemap: Arc<Texture>,
    pub output_cubemap: Arc<Texture>,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        SHADER_PATH
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
        render_context: &RenderContext,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let cubemap_output_view = desc
            .output_cubemap
            .array_view
            .as_ref()
            .expect("Output view missing");

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&desc.input_cubemap.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&cubemap_output_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&render_context.common_sampler),
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

    pub fn process(
        &self,
        source_texture: &Arc<Texture>,
        irradiance_resolution: u32,
    ) -> Result<Arc<Texture>, anyhow::Error> {
        let compute_task_factory = ComputeTaskFactory::new(self.render_env);
        let result_texture = Arc::new(
            TextureBuilder::new(
                &self.render_env.render_context.device,
                &self.render_env.render_context.queue,
            )
            .with_label("diffuse_irradiance")
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_size(irradiance_resolution, irradiance_resolution)
            .as_cubemap()
            .with_usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
            )
            .build(),
        );
        let diffuse_task = compute_task_factory.create_task::<Task>(TaskDescriptor {
            input_cubemap: Arc::clone(&source_texture),
            output_cubemap: Arc::clone(&result_texture),
        });

        compute_task_factory
            .create_executor()
            .record(&self.render_env.render_context, &diffuse_task)
            .execute(&self.render_env.render_context)
            .wait(&self.render_env.render_context);
        Ok(result_texture)
    }
}
