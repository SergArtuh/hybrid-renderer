use std::{cell::Cell, sync::Arc};

use anyhow::Context;
use wgpu::util::DeviceExt;

use crate::{
    core::{
        material::{
            MaterialDomain, MaterialTrait, MaterialType, SpriteMaterial, SpriteSheetUniform,
        },
        render_context::RenderContext,
        texture::Texture,
        vertex::Vertex,
    },
    renderer::materials::{DEFAULT_DEPTH_FORMAT, PipelineVisitorEnvironment},
};

pub struct SpriteMaterialDescriptor {
    pub texture: Arc<Texture>,
    pub grid_size: (u32, u32),
}

impl MaterialTrait for SpriteMaterial {
    type Descriptor = SpriteMaterialDescriptor;
    const DOMAIN: MaterialDomain = MaterialDomain::Surface;
    const TYPE: MaterialType = MaterialType::Sprite;
    fn create(
        context: &RenderContext,
        desc: Self::Descriptor,
        //TODO: get layout from MaterialTrait
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, anyhow::Error> {
        let material_definition = SpriteMaterialDefinition::default();
        let material = material_definition.create_instance(context, desc, layout);
        Ok(material)
    }

    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>] {
        const LAYOUT: &[wgpu::VertexBufferLayout<'static>] = &[Vertex::LAYOUT];
        LAYOUT
    }
}
#[derive(Default, Clone)]
pub struct SpriteMaterialDefinition;

impl SpriteMaterialDefinition {
    pub fn create_instance(
        &self,
        render_context: &RenderContext,
        desc: SpriteMaterialDescriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> SpriteMaterial {
        let texture = Arc::clone(&desc.texture.view);

        let config = SpriteSheetUniform {
            config: [desc.grid_size.0 as f32, desc.grid_size.1 as f32, 0.0, 0.0],
        };

        let uniform_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Sprite Animation Buffer"),
                    contents: bytemuck::cast_slice(&[config]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &render_context.common_nearest_sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            uniform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: Some("Sprite material bind group"),
            });

        SpriteMaterial {
            texture,
            config: Cell::new(config),
            uniform_buffer,
            bind_group,
        }
    }
}

impl SpriteMaterialDefinition {
    pub fn create_pipeline(
        environment: &mut PipelineVisitorEnvironment<'_>,
    ) -> Result<wgpu::RenderPipeline, anyhow::Error> {
        let render_context = environment.context;
        let pipeline_definition = environment.pipeline_definition;
        render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_path = &pipeline_definition.shader_path;
        let source = std::fs::read_to_string(shader_path)
            .with_context(|| format!("Failed to read shader file at {:?}", shader_path))?;

        let label = shader_path
            .to_str()
            .context("Shader path contains invalid UTF-8 characters")?;

        let material =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Sprite material bind group layout"),
                });

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &environment.layout.global,
                        &environment.layout.model,
                        &material,
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
                    //depth_stencil: None,
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEFAULT_DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        environment
            .layout
            .pipeline_layouts
            .insert(pipeline_definition.material_type, pipeline_layout);

        environment
            .layout
            .materials
            .insert(pipeline_definition.material_type, Arc::new(material));

        Ok(pipeline)
    }
}
