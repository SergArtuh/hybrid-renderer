use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use wgpu::util::DeviceExt;

use crate::{
    core::{
        material::{
            EnvironmentMap, MaterialDomain, MaterialTrait, MaterialType, SkydomeEnvironmentMaterial,
        },
        texture::Texture,
        uniforms::SkydomeUniform,
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

pub struct Descriptor {
    pub skybox_texture: Arc<Texture>,
    pub irradiance_texture: Arc<Texture>,
    pub specular_texture: Arc<Texture>,
    pub dome_radius: f32,
    pub dome_factor: f32,
}

impl MaterialTrait for SkydomeEnvironmentMaterial {
    type Descriptor = Descriptor;
    type Definition = Definition;
    const DOMAIN: MaterialDomain = MaterialDomain::Environment;
    const TYPE: MaterialType = MaterialType::Skydome;
    const LAYOUT: &'static [wgpu::VertexBufferLayout<'static>] = &[Vertex::LAYOUT];

    fn get_shader_path() -> &'static str {
        "skydome.wgsl"
    }
}
#[derive(Default, Clone)]
pub struct Definition;

impl MaterialHasDefinition for SkydomeEnvironmentMaterial {
    type Def = Definition;
}

impl MaterialDefinitionTrait<SkydomeEnvironmentMaterial> for Definition {
    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> SkydomeEnvironmentMaterial {
        let skydome_uniform = SkydomeUniform::new(desc.dome_radius, desc.dome_factor);

        let uniform_buffer = render_env.render_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Skydome Uniform Buffer"),
                contents: bytemuck::cast_slice(&[skydome_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group =
            render_env
                .render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            uniform_buffer.as_entire_buffer_binding(),
                        ),
                    }],
                    label: Some("Skybox material bind group"),
                });

        SkydomeEnvironmentMaterial {
            environment_map: EnvironmentMap {
                skybox: Arc::clone(&desc.skybox_texture.view),
                irradiance: Arc::clone(&desc.irradiance_texture.view),
                specular: Arc::clone(&desc.specular_texture.view),
            },
            uniform_buffer,
            bind_group,
        }
    }
    // }

    // impl SkydomeMaterialDefinition {
    fn create_pipeline(
        render_env: &RenderingEnvironment,
    ) -> Result<MaterialPipelineResult, anyhow::Error> {
        let render_context = &render_env.render_context;
        let pipeline_definition = SkydomeEnvironmentMaterial::get_definition(); //environment.pipeline_definition;
        render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_file_path = &pipeline_definition.shader_path;

        let base_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders")
            .join("surface");

        let shader_path = base_shaders_path.clone().join(shader_file_path);
        let source = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Failed to read shader file at {:?}", &shader_path))?;

        let label = shader_path
            .to_str()
            .context("Shader path contains invalid UTF-8 characters")?;

        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Skydome material bind group layout"),
                });

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &render_env.pipeline_resources.global,
                        &render_env.pipeline_resources.model,
                        &bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let shader_module =
            render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });

        let pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: render_context.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEFAULT_DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Ok(MaterialPipelineResult {
            bind_group_layout: Arc::new(bind_group_layout),
            render_pipeline: pipeline,
            pipeline_layout,
        })
    }
}
