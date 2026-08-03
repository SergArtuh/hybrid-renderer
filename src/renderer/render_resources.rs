use std::sync::Arc;

use crate::{
    core::{
        render_context::RenderContext,
        texture::DefaultTextures,
        uniforms::{CameraUniform, ModelUniform},
    },
    renderer::{
        passes::forward::ForwardPass, pipeline_resources::PipelineResources,
        pipeline_system::PipelineSystem,
    },
    stage::frame_data::RenderItem,
};

pub struct RenderResources {
    pub forward_pass: ForwardPass,
    pub global_resources: GlobalResources,
    pub model_resources: ModelResources,
    pub depth_texture_view: wgpu::TextureView,
    pub default_textures: DefaultTextures,
    pub common_sampler: wgpu::Sampler,
    pub common_nearest_sampler: wgpu::Sampler,
}

impl RenderResources {
    pub fn new(render_context: &RenderContext, pipeline_resources: &PipelineResources) -> Self {
        render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let default_textures = DefaultTextures::new(&render_context.device, &render_context.queue);
        let common_sampler = render_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
        let common_nearest_sampler =
            render_context
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let global_resources = GlobalResources::new(
            render_context,
            &default_textures,
            &common_sampler,
            &pipeline_resources.global,
        );
        let model_resources = ModelResources::new(render_context, &pipeline_resources.model);
        let depth_texture_view = Self::create_depth_texture(render_context);

        let forward_pass = ForwardPass::default();

        Self {
            forward_pass,
            global_resources,
            model_resources,
            depth_texture_view,
            default_textures,
            common_sampler,
            common_nearest_sampler,
        }
    }

    pub fn resize(&mut self, render_context: &RenderContext) {
        self.depth_texture_view = Self::create_depth_texture(render_context);
    }

    fn create_depth_texture(render_context: &RenderContext) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: render_context.config.width,
            height: render_context.config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PipelineSystem::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = render_context.device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

pub struct GlobalResources {
    pub camera_uniform_buffer: wgpu::Buffer,
    pub skybox_texture: Arc<wgpu::TextureView>,
    pub bind_group: wgpu::BindGroup,
}

impl GlobalResources {
    pub fn new(
        render_context: &RenderContext,
        default_textures: &DefaultTextures,
        common_sampler: &wgpu::Sampler,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let camera_uniform_buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: std::mem::size_of::<CameraUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let skybox_texture = Arc::clone(&default_textures.cubemap.view);

        let irradiance_texture = Arc::clone(&default_textures.cubemap.view);

        let specular_texture = Arc::clone(&default_textures.cubemap.view);

        let bind_group = Self::create_global_bind_group(
            render_context,
            bind_group_layout,
            &camera_uniform_buffer,
            &skybox_texture,
            &irradiance_texture,
            &specular_texture,
            &common_sampler,
        );

        Self {
            camera_uniform_buffer,
            skybox_texture,
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
        common_sampler: &wgpu::Sampler,
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
            &common_sampler,
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

    pub fn update_buffer(
        &self,
        render_context: &RenderContext,
        items: &[RenderItem],
        start_index: usize,
    ) {
        if items.is_empty() {
            return;
        }

        let mut data = Vec::with_capacity(items.len() * self.stride as usize);

        for item in items {
            let model_uniform = Self::make_model_uniform(*item.world_matrix());
            let bytes = bytemuck::bytes_of(&model_uniform);
            data.extend_from_slice(bytes);

            let padding = self.stride as usize - std::mem::size_of::<ModelUniform>();
            if padding > 0 {
                data.extend(std::iter::repeat(0).take(padding));
            }
        }

        // Вычисляем байтовое смещение в GPU-буфере
        let buffer_offset = (start_index as u64) * (self.stride as u64);

        render_context
            .queue
            .write_buffer(&self.buffer, buffer_offset, &data);
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
