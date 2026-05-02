use std::{collections::HashMap, sync::Arc};

use crate::core::{
    compute_task::ComputeTaskType, material::MaterialType, render_context::RenderContext,
    uniforms::CameraUniform,
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
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
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
    //pub skybox_texture: wgpu::TextureView,
    //pub skybox_sampler: wgpu::Sampler,
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

        let bind_group = Self::create_global_bind_group(
            render_context,
            bind_group_layout,
            &camera_uniform_buffer,
        );

        Self {
            camera_uniform_buffer,
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

    fn create_global_bind_group(
        render_context: &RenderContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        camera_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let camera_entries = Self::get_camera_entries(camera_uniform_buffer);
        render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &camera_entries,
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
}
