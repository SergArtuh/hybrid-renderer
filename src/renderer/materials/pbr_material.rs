use std::sync::Arc;

use anyhow::Context;
use wgpu::util::DeviceExt;

use crate::{
    core::{
        material::{MaterialDomain, MaterialTrait, MaterialType, PhysicalMaterial},
        render_context::RenderContext,
        texture::Texture,
        uniforms::PbrMaterialUniforms,
        vertex::Vertex,
    },
    renderer::materials::{DEFAULT_DEPTH_FORMAT, PipelineVisitorEnvironment},
};

pub struct PhysicalMaterialDescriptor {
    pub base_color: Option<Arc<Texture>>,
    pub normal: Option<Arc<Texture>>,
    pub metallic_roughness: Option<Arc<Texture>>,
    pub occlusion: Option<Arc<Texture>>,
    pub emissive: Option<Arc<Texture>>,
    pub base_color_factor: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub normal_scale: f32,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub occlusion_strength: f32,
    pub clearcoat_factor: f32,
    pub clearcoat_roughness: f32,
    pub clearcoat_texture: Option<Arc<Texture>>,
    pub clearcoat_roughness_texture: Option<Arc<Texture>>,
    pub clearcoat_normal_texture: Option<Arc<Texture>>,
}

impl MaterialTrait for PhysicalMaterial {
    type Descriptor = PhysicalMaterialDescriptor;
    const DOMAIN: MaterialDomain = MaterialDomain::Surface;
    const TYPE: MaterialType = MaterialType::Physical;
    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>] {
        const LAYOUT: &[wgpu::VertexBufferLayout<'static>] = &[Vertex::LAYOUT];
        LAYOUT
    }

    fn create(
        context: &RenderContext,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, anyhow::Error> {
        let material_definition = PbrMaterialDefinition::default();
        let material = material_definition.create_instance(context, desc, layout);
        Ok(material)
    }
}

#[derive(Default, Clone)]
pub struct PbrMaterialDefinition;

impl PbrMaterialDefinition {
    pub fn create_instance(
        &self,
        render_context: &RenderContext,
        desc: PhysicalMaterialDescriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> PhysicalMaterial {
        let base_color = desc
            .base_color
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let normal = desc
            .normal
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.normal));

        let metallic_roughness = desc
            .metallic_roughness
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.black_metallic));

        let emissive = desc
            .emissive
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.black));

        let occlusion = desc
            .occlusion
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let clearcoat = desc
            .clearcoat_texture
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let clearcoat_roughness = desc
            .clearcoat_roughness_texture
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let clearcoat_normal = desc
            .clearcoat_normal_texture
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let uniform = PbrMaterialUniforms {
            base_color_factor: desc.base_color_factor,
            emissive_and_scale: [
                desc.emissive_factor[0],
                desc.emissive_factor[1],
                desc.emissive_factor[2],
                desc.normal_scale,
            ],
            pbr_factors: [
                desc.roughness_factor,
                desc.metallic_factor,
                desc.occlusion_strength,
                desc.clearcoat_factor,
            ],
            clearcoat_factors: [desc.clearcoat_roughness, 0.0, 0.0, 0.0],
        };

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                label: Some("Physical Material Bind Group"),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&base_color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&normal),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&metallic_roughness),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&occlusion),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&emissive),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&render_context.common_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &render_context.device.create_buffer_init(
                                &wgpu::util::BufferInitDescriptor {
                                    label: Some("PbrMaterialUniforms"),
                                    contents: bytemuck::cast_slice(&[uniform]),
                                    usage: wgpu::BufferUsages::UNIFORM
                                        | wgpu::BufferUsages::COPY_DST,
                                },
                            ),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(&clearcoat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(&clearcoat_roughness),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(&clearcoat_normal),
                    },
                ],
            });

        PhysicalMaterial {
            base_color,
            normal,
            metallic_roughness,
            occlusion,
            emissive,
            bind_group,
        }
    }
}

impl PbrMaterialDefinition {
    pub fn create_pipeline(
        environment: &PipelineVisitorEnvironment<'_>,
    ) -> Result<wgpu::RenderPipeline, anyhow::Error> {
        let mut interface = environment.layout.borrow_mut();
        let render_context = environment.context;
        let pipeline_definition = environment.pipeline_definition;

        let material =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("material_bind_group_layout"),
                    entries: &[
                        // Binding 0: Base Color (Diffuse)
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
                        // Binding 1: Normal Map
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 2: Metallic-Roughness Map
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 3: Occlusion Map
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 4: Emissive Map
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 5: Common Sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // Binding 6: PBR Material Uniforms
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Binding 7: Clearcoat
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 8: Clearcoat Roughness
                        wgpu::BindGroupLayoutEntry {
                            binding: 8,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Binding 9: Clearcoat Normal
                        wgpu::BindGroupLayoutEntry {
                            binding: 9,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Phisical Material Pipeline Layout"),
                    bind_group_layouts: &[&interface.global, &interface.model, &material],
                    push_constant_ranges: &[],
                });

        render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_path = &pipeline_definition.shader_path;
        let source = std::fs::read_to_string(shader_path)
            .with_context(|| format!("Failed to read shader file at {:?}", shader_path))?;

        let label = shader_path
            .to_str()
            .context("Shader path contains invalid UTF-8 characters")?;

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
                        buffers: (pipeline_definition.layout_fetcher)(),
                        compilation_options: Default::default(),
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEFAULT_DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
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
