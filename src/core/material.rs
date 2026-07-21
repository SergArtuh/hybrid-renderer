use std::{cell::Cell, path::PathBuf, sync::Arc};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum MaterialType {
    Sprite,
    Physical,
    Skydome,
}

pub enum MaterialDomain {
    Surface,
    PostProcess,
    Environment,
}

pub enum Material {
    Sprite(SpriteMaterial),
    Physical(PhysicalMaterial),
    Skydome(SkydomeEnvironmentMaterial),
}

impl Material {
    pub fn kind(&self) -> MaterialType {
        match self {
            Material::Sprite(_) => MaterialType::Sprite,
            Material::Physical(_) => MaterialType::Physical,
            Material::Skydome(_) => MaterialType::Skydome,
        }
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        match self {
            Material::Sprite(sprite_material) => &sprite_material.bind_group,
            Material::Physical(physical_material) => &physical_material.bind_group,
            Material::Skydome(skydome_material) => &skydome_material.bind_group,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub alpha_mode: AlphaMode,
}

impl Default for PipelineKey {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlphaMode {
    Opaque,
    Blend,
}

pub trait MaterialTrait: Sized {
    type Descriptor;
    type Definition;
    const TYPE: MaterialType;
    const DOMAIN: MaterialDomain;
    const LAYOUT: &[wgpu::VertexBufferLayout<'static>];

    fn get_shader_path() -> &'static str;

    fn get_definition() -> MaterialDefinition {
        MaterialDefinition {
            shader_path: Self::get_shader_path().into(),
            material_type: Self::TYPE,
            layout: Self::LAYOUT,
        }
    }

    fn supported_keys() -> Vec<PipelineKey> {
        vec![PipelineKey::default()]
    }
}

#[derive(Clone, Debug)]
pub struct MaterialDefinition {
    pub shader_path: PathBuf,
    pub material_type: MaterialType,
    pub layout: &'static [wgpu::VertexBufferLayout<'static>],
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

pub struct EnvironmentMap {
    pub skybox: Arc<wgpu::TextureView>,
    pub irradiance: Arc<wgpu::TextureView>,
    pub specular: Arc<wgpu::TextureView>,
}

pub struct SkydomeEnvironmentMaterial {
    pub environment_map: EnvironmentMap,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
