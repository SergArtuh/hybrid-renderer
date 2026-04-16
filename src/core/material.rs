use crate::core::render_context::RenderContext;
use std::{cell::Cell, sync::Arc};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum MaterialType {
    Sprite,
    Physical,
    Skybox,
}

pub enum MaterialDomain {
    Surface,
    PostProcess,
    Environment,
}

pub enum Material {
    Sprite(SpriteMaterial),
    Physical(PhysicalMaterial),
    Skybox(SkyboxEnvironmentMaterial),
}

impl Material {
    pub fn kind(&self) -> MaterialType {
        match self {
            Material::Sprite(_) => MaterialType::Sprite,
            Material::Physical(_) => MaterialType::Physical,
            Material::Skybox(_) => MaterialType::Skybox,
        }
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        match self {
            Material::Sprite(sprite_material) => &sprite_material.bind_group,
            Material::Physical(physical_material) => &physical_material.bind_group,
            Material::Skybox(skybox_material) => &skybox_material.bind_group,
        }
    }
}

pub trait MaterialTrait: Sized {
    type Descriptor;
    const TYPE: MaterialType;
    const DOMAIN: MaterialDomain;
    fn get_layout() -> &'static [wgpu::VertexBufferLayout<'static>];
    // Метод создания "живого" материала из описания
    fn create(
        _context: &RenderContext,
        _desc: Self::Descriptor,
        _layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, anyhow::Error> {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteSheetUniform {
    // [grid_columns, grid_rows, current_frame_index, _padding]
    pub config: [f32; 4],
}
pub struct SpriteMaterial {
    pub texture: Arc<wgpu::TextureView>,
    pub config: Cell<SpriteSheetUniform>,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl SpriteMaterial {
    pub fn set_frame(&self, frame_index: u32, queue: &wgpu::Queue) {
        let mut config = self.config.get();
        config.config[2] = frame_index as f32;
        self.config.set(config);
        self.update(queue);
    }

    fn update(&self, queue: &wgpu::Queue) {
        let config = self.config.get();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[config]));
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

pub struct SkyboxEnvironmentMaterial {
    pub texture: Arc<wgpu::TextureView>,
    pub bind_group: wgpu::BindGroup,
}
