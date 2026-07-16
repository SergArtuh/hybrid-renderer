use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::core::{
    compute_task::ComputeTaskType, material::MaterialType, render_context::RenderContext,
    uniforms::ModelUniform,
};

pub struct PipelineResources {
    pub global: wgpu::BindGroupLayout,
    pub model: wgpu::BindGroupLayout,
    pub materials: HashMap<MaterialType, Arc<wgpu::BindGroupLayout>>,
    pub compute_tasks: HashMap<ComputeTaskType, Arc<wgpu::BindGroupLayout>>,
    pub pipeline_layouts: HashMap<MaterialType, wgpu::PipelineLayout>,
    pub compute_pipeline_layouts: HashMap<ComputeTaskType, wgpu::PipelineLayout>,
    pub render_pipelines: HashMap<MaterialType, wgpu::RenderPipeline>,
    pub compute_pipelines: HashMap<ComputeTaskType, wgpu::ComputePipeline>,
    pub base_shaders_path: PathBuf,
    pub base_compute_shaders_path: PathBuf,
}

impl PipelineResources {
    pub fn new(render_context: &RenderContext) -> Self {
        let global =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                    ],
                    label: Some("global_bind_group_layout"),
                });

        let model =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<ModelUniform>() as _,
                            ),
                            // has_dynamic_offset: false,
                            // min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("model_layout"),
                });

        Self {
            global,
            model,
            materials: HashMap::new(),
            compute_tasks: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            compute_pipeline_layouts: HashMap::new(),
            render_pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            base_shaders_path: PathBuf::new(),
            base_compute_shaders_path: PathBuf::new(),
        }
    }
}
