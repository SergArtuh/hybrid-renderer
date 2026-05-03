use std::{collections::HashMap, sync::Arc};

use crate::{
    core::{
        compute_task::ComputeTaskType, material::MaterialType, render_context::RenderContext,
        uniforms::CameraUniform,
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
                            min_binding_size: wgpu::BufferSize::new(64),
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

        let bind_group = Self::create_global_bind_group(
            render_context,
            bind_group_layout,
            &camera_uniform_buffer,
            &skybox_texture,
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
            &render_context.common_sampler,
        );
    }

    fn create_global_bind_group(
        render_context: &RenderContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        camera_uniform_buffer: &wgpu::Buffer,
        skybox_texture: &wgpu::TextureView,
        skybox_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let mut entries = Self::get_camera_entries(camera_uniform_buffer);
        entries.extend(Self::get_skydome_entries(&skybox_texture, &skybox_sampler));

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
        texture: &'a wgpu::TextureView,
        sampler: &'a wgpu::Sampler,
    ) -> Vec<wgpu::BindGroupEntry<'a>> {
        vec![
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
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
        let matrix_size = std::mem::size_of::<glam::Mat4>() as wgpu::BufferAddress;
        let alignment = render_context
            .device
            .limits()
            .min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let dynamic_offset_step = (matrix_size + alignment - 1) & !(alignment - 1);

        let buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Global Model Matrix Buffer"),
                size: dynamic_offset_step * Self::MAX_MODELS,
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
                        size: wgpu::BufferSize::new(matrix_size),
                    }),
                }],
                label: Some("model_bind_group"),
            });

        let stride = Self::calculate_stride(render_context);

        Self {
            buffer,
            bind_group,
            stride,
        }
    }
    pub fn update_buffer(&self, render_context: &RenderContext, items: &[RenderItem]) {
        let matrix_size = std::mem::size_of::<glam::Mat4>();

        let mut data = Vec::with_capacity(items.len() * self.stride as usize);

        for item in items {
            let matrix = item.world_matrix();
            let bytes = bytemuck::cast_slice(matrix.as_ref());
            data.extend_from_slice(bytes);
            let padding = self.stride as usize - matrix_size;
            data.extend(std::iter::repeat(0).take(padding));
        }

        render_context.queue.write_buffer(&self.buffer, 0, &data);
    }

    fn calculate_stride(render_context: &RenderContext) -> wgpu::BufferAddress {
        let matrix_size = std::mem::size_of::<glam::Mat4>();
        let alignment = render_context
            .device
            .limits()
            .min_uniform_buffer_offset_alignment as usize;
        let stride: usize = (matrix_size + alignment - 1) & !(alignment - 1);
        stride as wgpu::BufferAddress
    }
}
