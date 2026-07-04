use wgpu::util::DeviceExt;

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
        render_context::RenderContext,
        texture::Texture,
        texture_builder::TextureBuilder,
        uniforms::SpecularPrefilterUniform,
    },
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};
use std::sync::Arc;

const TASK_TYPE: ComputeTaskType = ComputeTaskType::SpecularPrefilter;
const SHADER_PATH: &str = "specular_prefilter.wgsl";

pub struct TaskDescriptor {
    pub input_cubemap: Arc<Texture>,
    pub output_cubemap: Arc<Texture>,
    pub config: Arc<wgpu::Buffer>, // TODO: make wraper
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
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
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(
                            desc.config.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: Some("Specular Prefilter Bind Group"),
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

    pub fn process(&self, source_cubemap: &Arc<Texture>) -> Result<Arc<Texture>, anyhow::Error> {
        let origin_width = source_cubemap.width;
        let origin_height = source_cubemap.height;

        let mip_count = (origin_width.max(origin_height) as f32).log2().floor() as u32 + 1;

        let result_texture = Arc::new(
            TextureBuilder::new(
                &self.render_env.render_context.device,
                &self.render_env.render_context.queue,
            )
            .with_label("final_mipmapped_cubemap")
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_size(origin_width, origin_height)
            .with_mip_level_count(mip_count)
            .as_cubemap()
            .with_usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .build(),
        );
        let compute_task_factory = ComputeTaskFactory::new(self.render_env);
        let mut executor = compute_task_factory.create_executor();

        for mip in 0..mip_count {
            let mip_width = (origin_width >> mip).max(1);
            let mip_height = (origin_height >> mip).max(1);
            let linear_roughness = mip as f32 / (mip_count - 1).max(1) as f32;
            let roughness = linear_roughness * linear_roughness;

            let temp_specular_mip_out = Arc::new(
                TextureBuilder::new(
                    &self.render_env.render_context.device,
                    &self.render_env.render_context.queue,
                )
                .with_label(&format!("temp_specular_mip_{}", mip))
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(mip_width, mip_height)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                )
                .build(),
            );

            let specular_mipmap_data = SpecularPrefilterUniform::new(roughness);
            let specular_mipmap_buffer =
                Arc::new(self.render_env.render_context.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Specular Prefilter Uniform Buffer"),
                        contents: bytemuck::cast_slice(&[specular_mipmap_data]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                ));

            let prefilter_task = compute_task_factory.create_task::<Task>(TaskDescriptor {
                input_cubemap: Arc::clone(&source_cubemap),
                output_cubemap: Arc::clone(&temp_specular_mip_out),
                config: Arc::clone(&specular_mipmap_buffer),
            });

            executor.record(&self.render_env.render_context, &prefilter_task);

            executor
                .get_encoder_mut()
                .expect("Encoder not found")
                .copy_texture_to_texture(
                    wgpu::ImageCopyTexture {
                        texture: &temp_specular_mip_out.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyTexture {
                        texture: &result_texture.texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: mip_width,
                        height: mip_height,
                        depth_or_array_layers: 6,
                    },
                );
        }
        executor.execute(&self.render_env.render_context);
        executor.wait(&self.render_env.render_context);
        Ok(result_texture)
    }
}
