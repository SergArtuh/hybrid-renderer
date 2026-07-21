pub mod pbr_material;
pub mod skydome_material;
pub mod sprite_material;

use std::path::PathBuf;
use std::sync::Arc;

pub use pbr_material::Definition as PbrMaterialDefinition;
pub use pbr_material::Descriptor as PhysicalMaterialDescriptor;

pub use skydome_material::Definition as SkydomeMaterialDefinition;
pub use skydome_material::Descriptor as SkydomeMaterialDescriptor;

pub use sprite_material::Definition as SpriteMaterialDefinition;
pub use sprite_material::Descriptor as SpriteMaterialDescriptor;

use crate::core::material::PipelineKey;
use crate::core::material::SkydomeEnvironmentMaterial;
use crate::core::material::SpriteMaterial;
use crate::core::material::{Material, MaterialTrait, PhysicalMaterial};
use crate::renderer::RenderingEnvironment;

pub fn initialize_materials(render_env: &mut RenderingEnvironment) {
    render_env.pipeline_resources.base_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("renderer")
        .join("shaders")
        .join("surface");

    macro_rules! register {
        ($($t:ty),* $(,)?) => {
            $( crate::renderer::pipeline_system::PipelineSystem::register_pipeline::<$t>(render_env);
            #[cfg(feature = "shader-hot-reload")]
            crate::util::shader_watcher::ShaderWatcherSystem::register_pipeline::<$t>(render_env);
         )*
        };
    }

    register![PhysicalMaterial, SkydomeEnvironmentMaterial, SpriteMaterial];
}

const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct MaterialFactory<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

pub struct MaterialPipelineResult {
    pub bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub render_pipelines: Vec<(PipelineKey, wgpu::RenderPipeline)>,
    pub pipeline_layout: wgpu::PipelineLayout,
}

pub trait MaterialHasDefinition: MaterialTrait {
    type Def: MaterialDefinitionTrait<Self>;
}

pub trait MaterialDefinitionTrait<M: MaterialTrait> {
    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: M::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> M
    where
        M: MaterialHasDefinition;
    fn create_pipeline(
        render_env: &RenderingEnvironment,
    ) -> Result<MaterialPipelineResult, anyhow::Error>;
}

impl<'a> MaterialFactory<'a> {
    pub fn new(render_env: &'a RenderingEnvironment<'a>) -> Self {
        Self { render_env }
    }

    pub fn create_material<M: MaterialTrait>(&self, desc: M::Descriptor) -> M
    where
        M: MaterialHasDefinition,
    {
        let pipeline_resources = &self.render_env.pipeline_resources;

        let bind_group_layout = pipeline_resources
            .materials
            .get(&M::TYPE)
            .expect("Layout not found for material type {M::TYPE}");

        <<M as MaterialHasDefinition>::Def>::create_instance(
            self.render_env,
            desc,
            bind_group_layout,
        )
    }

    pub fn create_pipeline<M: MaterialTrait>(self) -> Result<MaterialPipelineResult, anyhow::Error>
    where
        M: MaterialHasDefinition,
    {
        <<M as MaterialHasDefinition>::Def>::create_pipeline(self.render_env)
    }

    pub fn create_default_material(&self) -> Material {
        Material::Physical(
            self.create_material::<PhysicalMaterial>(PhysicalMaterialDescriptor {
                base_color: None,
                normal: None,
                metallic_roughness: None,
                occlusion: None,
                emissive: None,
                base_color_factor: [0.0; 4],
                emissive_factor: [0.0; 3],
                normal_scale: 1.0,
                roughness_factor: 1.0,
                metallic_factor: 1.0,
                occlusion_strength: 1.0,
                clearcoat_factor: 0.0,
                clearcoat_roughness: 0.0,
                clearcoat_texture: None,
                clearcoat_roughness_texture: None,
                clearcoat_normal_texture: None,
            }),
        )
    }
}
