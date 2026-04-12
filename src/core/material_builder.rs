#![deprecated]

use std::{cell::Cell, sync::Arc};

use wgpu::util::DeviceExt;

use crate::core::{
    material::{PhysicalMaterial, SpriteMaterial, SpriteSheetUniform},
    render_context::RenderContext,
    texture::Texture,
};

pub struct PhysicalMaterialBuilder {
    base_color: Option<Arc<Texture>>,
    normal: Option<Arc<Texture>>,
    metallic_roughness: Option<Arc<Texture>>,
    occlusion: Option<Arc<Texture>>,
    emissive: Option<Arc<Texture>>,
}

impl PhysicalMaterialBuilder {
    pub fn new() -> Self {
        Self {
            base_color: None,
            normal: None,
            metallic_roughness: None,
            occlusion: None,
            emissive: None,
        }
    }

    pub fn with_base_color(mut self, base_color: Arc<Texture>) -> Self {
        self.base_color = Some(base_color);
        self
    }

    pub fn with_normal(mut self, normal: Arc<Texture>) -> Self {
        self.normal = Some(normal);
        self
    }

    pub fn with_metallic_roughness(mut self, metallic_roughness: Arc<Texture>) -> Self {
        self.metallic_roughness = Some(metallic_roughness);
        self
    }

    pub fn with_occlusion(mut self, occlusion: Arc<Texture>) -> Self {
        self.occlusion = Some(occlusion);
        self
    }

    pub fn with_emissive(mut self, emissive: Arc<Texture>) -> Self {
        self.emissive = Some(emissive);
        self
    }

    pub fn build(
        self,
        render_context: &RenderContext,
        layout: &wgpu::BindGroupLayout,
    ) -> PhysicalMaterial {
        let base_color = self
            .base_color
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

        let normal = self
            .normal
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.normal));

        let metallic_roughness = self
            .metallic_roughness
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.black_metallic));

        let emissive = self
            .emissive
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.black));

        let occlusion = self
            .occlusion
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| Arc::clone(&render_context.default_textures.white));

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

pub struct SpriteMaterialBuilder {
    texture: Option<Arc<Texture>>,
    rows: u32,
    columns: u32,
}

impl SpriteMaterialBuilder {
    pub fn new() -> Self {
        Self {
            texture: None,
            rows: 1,
            columns: 1,
        }
    }

    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_grid_size(mut self, rows: u32, columns: u32) -> Self {
        self.rows = rows;
        self.columns = columns;
        self
    }

    pub fn build(
        self,
        render_context: &RenderContext,
        layout: &wgpu::BindGroupLayout,
    ) -> SpriteMaterial {
        let texture = self
            .texture
            .map(|t| Arc::clone(&t.view))
            .unwrap_or_else(|| {
                println!("Texture is not defined. Using white texture for sprite material");
                Arc::clone(&render_context.default_textures.white)
            });

        let config = SpriteSheetUniform {
            config: [self.columns as f32, self.rows as f32, 0.0, 0.0],
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
