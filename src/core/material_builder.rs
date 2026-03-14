use std::sync::Arc;

use crate::core::{material::PhysicalMaterial, render_context::RenderContext, texture::Texture};

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
