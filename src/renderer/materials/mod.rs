pub mod pbr_material;
pub mod skydome_material;
pub mod sprite_material;

pub use pbr_material::PbrMaterialDefinition;
pub use skydome_material::SkydomeMaterialDescriptor;
pub use sprite_material::SpriteMaterialDefinition;

use crate::core::material::{Material, MaterialTrait, PhysicalMaterial};
use crate::core::render_context::RenderContext;
use crate::renderer::RenderingEnvironment;
use crate::renderer::layout_interface::LayoutInterface;
use crate::renderer::materials::pbr_material::PhysicalMaterialDescriptor;
use crate::renderer::pipeline_manager::PipelineDefinition;

const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub struct PipelineVisitorEnvironment<'a> {
    pub(crate) pipeline_definition: &'a PipelineDefinition,
    pub context: &'a RenderContext<'a>,
    pub layout: &'a mut LayoutInterface,
}

pub struct MaterialFactory<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

impl<'a> MaterialFactory<'a> {
    pub fn new(render_env: &'a RenderingEnvironment) -> Self {
        Self { render_env }
    }

    pub fn create_material<M: MaterialTrait>(&self, desc: M::Descriptor) -> M {
        let layout_interface = &self.render_env.layout_interface;
        let render_context = &self.render_env.render_context;

        let bind_group_layout = layout_interface
            .materials
            .get(&M::TYPE)
            .expect("Layout not found for material type {M::TYPE}");
        M::create(render_context, desc, &bind_group_layout)
            .expect("Failed to create material instance")
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
