use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::vertex::Vertex;

use super::texture::Texture;
use super::texture_builder::TextureBuilder;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum MaterialType {
    Sprite,
    Physical,
}

pub enum Material {
    Sprite(SpriteMaterial),
    Physical(PhysicalMaterial),
}

impl Material {
    pub fn kind(&self) -> MaterialType {
        match self {
            Material::Sprite(_) => MaterialType::Sprite,
            Material::Physical(_) => MaterialType::Physical,
        }
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        match self {
            Material::Sprite(_sprite_material) => todo!(),
            Material::Physical(physical_material) => &physical_material.bind_group,
        }
    }
}

pub trait MaterialTrait {
    const TYPE: MaterialType;
    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>];
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteSheetUniform {
    // [grid_columns, grid_rows, current_frame_index, _padding]
    config: [f32; 4],
}

pub struct SpriteMaterial {
    pub texture: Texture,
    config: SpriteSheetUniform,
    pub uniform_buffer: wgpu::Buffer,
}

impl MaterialTrait for SpriteMaterial {
    const TYPE: MaterialType = MaterialType::Sprite;
    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>] {
        &[]
    }
}

impl SpriteMaterial {
    pub fn new(
        atlas_texture_bytes: &[u8],
        rows: u32,
        columns: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture = TextureBuilder::new(device, queue)
            .from_bytes(atlas_texture_bytes)
            .with_size(columns, rows)
            //.with_format(wgpu::TextureFormat::Rgba32Float, false)
            .with_filter(wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
            .build();

        let config = SpriteSheetUniform {
            config: [columns as f32, rows as f32, 0.0, 0.0],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Animation Buffer"),
            contents: bytemuck::cast_slice(&[config]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            texture,
            config,
            uniform_buffer,
        }
    }

    pub fn set_frame(&mut self, frame_index: u32, queue: &wgpu::Queue) {
        self.config.config[2] = frame_index as f32;
        self.update(queue);
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.config]),
        );
    }
}

pub struct PhysicalMaterial {
    pub base_color: Arc<wgpu::TextureView>,
    pub normal: Arc<wgpu::TextureView>,
    pub metallic_roughness: Arc<wgpu::TextureView>,
    pub occlusion: Arc<wgpu::TextureView>,
    pub emissive: Arc<wgpu::TextureView>,
    pub bind_group: wgpu::BindGroup,
}

impl MaterialTrait for PhysicalMaterial {
    const TYPE: MaterialType = MaterialType::Physical;
    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>] {
        const LAYOUT: &[wgpu::VertexBufferLayout<'static>] = &[Vertex::LAYOUT];
        LAYOUT
    }
}
