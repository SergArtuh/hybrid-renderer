use std::{path::PathBuf, sync::Arc};

use anyhow::Context;

use crate::{
    core::{
        material::{AlphaMode, MaterialTrait, MaterialType, PipelineKey, PostProcessMaterial},
        vertex::Vertex,
    },
    renderer::{
        RenderingEnvironment,
        materials::{MaterialDefinitionTrait, MaterialHasDefinition, MaterialPipelineResult},
    },
};

pub struct Descriptor {
    pub normal_buffer_view: wgpu::TextureView,
    pub depth_buffer_view: wgpu::TextureView,
}

impl MaterialTrait for PostProcessMaterial {
    type Descriptor = Descriptor;
    type Definition = Definition;
    //const DOMAIN: MaterialDomain = MaterialDomain::Surface;
    const TYPE: MaterialType = MaterialType::PostProcess;
    const LAYOUT: &'static [Option<wgpu::VertexBufferLayout<'static>>] = &[Some(Vertex::LAYOUT)];

    fn get_shader_path() -> &'static str {
        "postprocess.wgsl"
    }
    fn supported_keys() -> Vec<PipelineKey> {
        vec![
            PipelineKey {
                alpha_mode: AlphaMode::Opaque,
            },
            PipelineKey {
                alpha_mode: AlphaMode::Blend,
            },
        ]
    }
}

#[derive(Default, Clone)]
pub struct Definition;

impl MaterialHasDefinition for PostProcessMaterial {
    type Def = Definition;
}

impl MaterialDefinitionTrait<PostProcessMaterial> for Definition {
    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> PostProcessMaterial {
        println!("Creating post process material");
        let bind_group =
            render_env
                .render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&desc.normal_buffer_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &render_env.render_resources.common_sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&desc.depth_buffer_view),
                        },
                    ],
                    label: Some("Post process material bind group"),
                });

        PostProcessMaterial { bind_group }
    }

    fn create_pipeline(
        render_env: &RenderingEnvironment,
    ) -> Result<MaterialPipelineResult, anyhow::Error> {
        let pipeline_definition = PostProcessMaterial::get_definition();
        let bind_group_layout = render_env.render_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("prepass_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            },
        );

        let pipeline_layout = render_env.render_context.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("prepass material pipeline layout"),
                bind_group_layouts: &[
                    Some(&render_env.pipeline_resources.global),
                    Some(&render_env.pipeline_resources.model),
                    Some(&bind_group_layout),
                ],
                immediate_size: 0,
            },
        );

        let _ = render_env
            .render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_file_path = &pipeline_definition.shader_path;

        let base_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders")
            .join("surface");

        let shader_path = base_shaders_path.clone().join(shader_file_path);

        let source = crate::renderer::shaders::load_shader_source(&shader_path)?;

        let label = shader_path
            .to_str()
            .context("Shader path contains invalid UTF-8 characters")?;

        let shader_module =
            render_env
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });

        let pipeline = render_env.render_context.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Prepass pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                depth_stencil: None,
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: render_env.render_context.config.format,
                        //blend: Some(wgpu::BlendState::REPLACE),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            },
        );

        Ok(MaterialPipelineResult {
            bind_group_layout: Arc::new(bind_group_layout),
            render_pipelines: vec![(PipelineKey::default(), pipeline)],
            pipeline_layout,
        })
    }
}
