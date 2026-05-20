use std::{collections::HashMap, sync::Arc};

use crate::{
    core::{
        compute_task::ComputeTaskType,
        material::MaterialType,
        render_context::RenderContext,
        uniforms::{CameraUniform, ModelUniform},
    },
    stage::frame_data::RenderItem,
};

pub struct LayoutInterface {
    pub global: wgpu::BindGroupLayout,
    pub model: wgpu::BindGroupLayout,
    pub materials: HashMap<MaterialType, Arc<wgpu::BindGroupLayout>>,
    pub compute_tasks: HashMap<ComputeTaskType, Arc<wgpu::BindGroupLayout>>,
    pub pipeline_layouts: HashMap<MaterialType, wgpu::PipelineLayout>,
    pub compute_pipeline_layouts: HashMap<ComputeTaskType, wgpu::PipelineLayout>,
}

impl LayoutInterface {
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
        }
    }
}

pub struct GlobalResources {
    pub camera_uniform_buffer: wgpu::Buffer,
    pub skybox_texture: Arc<wgpu::TextureView>,
    //pub skybox_sampler: Arc<wgpu::Sampler>,
    pub bind_group: wgpu::BindGroup,
}

impl GlobalResources {
    pub fn new(render_context: &RenderContext, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let camera_uniform_buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: std::mem::size_of::<CameraUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let skybox_texture = Arc::clone(&render_context.default_textures.cubemap.view);

        let irradiance_texture = Arc::clone(&render_context.default_textures.cubemap.view);

        let specular_texture = Arc::clone(&render_context.default_textures.cubemap.view);

        let bind_group = Self::create_global_bind_group(
            render_context,
            bind_group_layout,
            &camera_uniform_buffer,
            &skybox_texture,
            &irradiance_texture,
            &specular_texture,
            &render_context.common_sampler,
        );

        Self {
            camera_uniform_buffer,
            skybox_texture,
            // skybox_sampler,
            bind_group,
        }
    }

    pub fn update_camera_uniform_buffer(
        &self,
        render_context: &RenderContext,
        camera_uniform: &CameraUniform,
    ) {
        render_context.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[*camera_uniform]),
        );
    }

    pub fn update_skybox_texture(
        &mut self,
        render_context: &RenderContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        skybox_texture: Arc<wgpu::TextureView>,
        irradiance_texture: Arc<wgpu::TextureView>,
        specular_texture: Arc<wgpu::TextureView>,
    ) {
        if Arc::ptr_eq(&self.skybox_texture, &skybox_texture) {
            return;
        }

        self.skybox_texture = Arc::clone(&skybox_texture);
        self.bind_group = Self::create_global_bind_group(
            render_context,
            bind_group_layout,
            &self.camera_uniform_buffer,
            &self.skybox_texture,
            &irradiance_texture,
            &specular_texture,
            &render_context.common_sampler,
        );
    }

    fn create_global_bind_group(
        render_context: &RenderContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        camera_uniform_buffer: &wgpu::Buffer,
        skybox_texture: &wgpu::TextureView,
        irradiance_texture: &wgpu::TextureView,
        specular_texture: &wgpu::TextureView,
        skybox_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let mut entries = Self::get_camera_entries(camera_uniform_buffer);
        entries.extend(Self::get_skydome_entries(
            &skybox_texture,
            &irradiance_texture,
            &specular_texture,
            &skybox_sampler,
        ));

        render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &entries,
                label: Some("global_bind_group"),
            })
    }

    fn get_camera_entries<'a>(
        camera_uniform_buffer: &'a wgpu::Buffer,
    ) -> Vec<wgpu::BindGroupEntry<'a>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(
                camera_uniform_buffer.as_entire_buffer_binding(),
            ),
        }]
    }
    fn get_skydome_entries<'a>(
        skybox_texture: &'a wgpu::TextureView,
        irradiance_texture: &'a wgpu::TextureView,
        specular_texture: &'a wgpu::TextureView,
        sampler: &'a wgpu::Sampler,
    ) -> Vec<wgpu::BindGroupEntry<'a>> {
        vec![
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&skybox_texture),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&irradiance_texture),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&specular_texture),
            },
        ]
    }
}
pub struct ModelResources {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) stride: wgpu::BufferAddress,
}

impl ModelResources {
    const MAX_MODELS: u64 = 100;
    pub fn new(render_context: &RenderContext, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let stride = Self::calculate_stride(render_context);
        let uniform_size = std::mem::size_of::<ModelUniform>() as wgpu::BufferAddress;

        let buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Model Uniform Buffer"),
                size: stride * Self::MAX_MODELS,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(uniform_size),
                    }),
                }],
                label: Some("model_bind_group"),
            });

        Self {
            buffer,
            bind_group,
            stride,
        }
    }

    pub fn update_buffer(&self, render_context: &RenderContext, items: &[RenderItem]) {
        let mut data = Vec::with_capacity(items.len() * self.stride as usize);

        for item in items {
            let model_uniform = Self::make_model_uniform(item.world_matrix().clone());
            let bytes = bytemuck::bytes_of(&model_uniform);
            data.extend_from_slice(bytes);

            let padding = self.stride as usize - std::mem::size_of::<ModelUniform>();
            if padding > 0 {
                data.extend(std::iter::repeat(0).take(padding));
            }
        }

        render_context.queue.write_buffer(&self.buffer, 0, &data);
    }

    fn calculate_stride(render_context: &RenderContext) -> wgpu::BufferAddress {
        let struct_size = std::mem::size_of::<ModelUniform>();

        let alignment = render_context
            .device
            .limits()
            .min_uniform_buffer_offset_alignment as usize;

        let stride = (struct_size + alignment - 1) & !(alignment - 1);

        stride as wgpu::BufferAddress
    }

    fn make_model_uniform(model_matrix: glam::Mat4) -> ModelUniform {
        let normal_matrix = model_matrix.clone().inverse().transpose();

        ModelUniform {
            model_matrix,
            normal_matrix,
        }
    }
}
