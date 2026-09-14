use std::{path::PathBuf, sync::Arc};

use anyhow::Context;

use crate::{
    core::{
        material::{AlphaMode, MaterialTrait, MaterialType, PipelineKey, PrePassMaterial},
        vertex::Vertex,
    },
    renderer::{
        RenderingEnvironment,
        materials::{
            DEFAULT_DEPTH_FORMAT, MaterialDefinitionTrait, MaterialHasDefinition,
            MaterialPipelineResult,
        },
    },
};

pub struct Descriptor {}

impl MaterialTrait for PrePassMaterial {
    type Descriptor = Descriptor;
    type Definition = Definition;
    //const DOMAIN: MaterialDomain = MaterialDomain::Surface;
    const TYPE: MaterialType = MaterialType::PrePass;
    const LAYOUT: &'static [Option<wgpu::VertexBufferLayout<'static>>] = &[Some(Vertex::LAYOUT)];

    fn get_shader_path() -> &'static str {
        "prepass.wgsl"
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

impl MaterialHasDefinition for PrePassMaterial {
    type Def = Definition;
}

impl MaterialDefinitionTrait<PrePassMaterial> for Definition {
    fn create_instance(
        _render_env: &RenderingEnvironment,
        _desc: Descriptor,
        _layout: &wgpu::BindGroupLayout,
    ) -> PrePassMaterial {
        unreachable!("PrePassMaterial is service material that cannot be created directly");
    }

    fn create_pipeline(
        render_env: &RenderingEnvironment,
    ) -> Result<MaterialPipelineResult, anyhow::Error> {
        let pipeline_definition = PrePassMaterial::get_definition(); // render_env.pipeline_definition;

        let bind_group_layout = render_env.render_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("prepass_bind_group_layout"),
                entries: &[],
            },
        );

        let pipeline_layout = render_env.render_context.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("prepass material pipeline layout"),
                bind_group_layouts: &[
                    Some(&render_env.pipeline_resources.global),
                    Some(&render_env.pipeline_resources.model),
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
                label: Some("Opaque Pbr Material Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: pipeline_definition.layout,
                    compilation_options: Default::default(),
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEFAULT_DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: render_env.render_context.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
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
