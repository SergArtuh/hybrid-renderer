use std::sync::Arc;

use anyhow::Context;

use crate::{
    core::{
        material::{MaterialDomain, MaterialTrait, MaterialType, SkyboxEnvironmentMaterial},
        render_context::RenderContext,
        texture::Texture,
        vertex::Vertex,
    },
    renderer::materials::{DEFAULT_DEPTH_FORMAT, PipelineVisitorEnvironment},
};

pub struct SkyboxMaterialDescriptor {
    pub texture: Arc<Texture>,
}

impl MaterialTrait for SkyboxEnvironmentMaterial {
    type Descriptor = SkyboxMaterialDescriptor;
    const DOMAIN: MaterialDomain = MaterialDomain::Environment;
    const TYPE: MaterialType = MaterialType::Skybox;
    fn create(
        context: &RenderContext,
        desc: Self::Descriptor,
        //TODO: get layout from MaterialTrait
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, anyhow::Error> {
        let material_definition = SkyboxMaterialDefinition::default();
        let material = material_definition.create_instance(context, desc, layout);
        Ok(material)
    }

    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>] {
        const LAYOUT: &[wgpu::VertexBufferLayout<'static>] = &[Vertex::LAYOUT];
        LAYOUT
    }
}
#[derive(Default, Clone)]
pub struct SkyboxMaterialDefinition;

impl SkyboxMaterialDefinition {
    pub fn create_instance(
        &self,
        render_context: &RenderContext,
        desc: SkyboxMaterialDescriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> SkyboxEnvironmentMaterial {
        let texture = Arc::clone(&desc.texture.view);

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
                ],
                label: Some("Sprite material bind group"),
            });

        SkyboxEnvironmentMaterial {
            texture,
            bind_group,
        }
    }
}

impl SkyboxMaterialDefinition {
    pub fn create_pipeline(
        environment: &PipelineVisitorEnvironment<'_>,
    ) -> Result<wgpu::RenderPipeline, anyhow::Error> {
        let mut interface = environment.layout.borrow_mut();
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
                    ],
                    label: Some("Sprite material bind group layout"),
                });

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&interface.global, &interface.model, &material],
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

        interface
            .pipeline_layouts
            .insert(pipeline_definition.material_type, pipeline_layout);

        interface
            .materials
            .insert(pipeline_definition.material_type, Arc::new(material));

        Ok(pipeline)
    }
}
