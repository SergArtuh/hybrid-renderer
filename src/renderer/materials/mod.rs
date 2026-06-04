pub mod pbr_material;
pub mod skydome_material;
pub mod sprite_material;

pub use pbr_material::PbrMaterialDefinition;
pub use skydome_material::SkydomeMaterialDescriptor;
pub use sprite_material::SpriteMaterialDefinition;

use crate::core::material::{Material, MaterialTrait, PhysicalMaterial};
use crate::core::render_context::RenderContext;
use crate::renderer::layout_interface::LayoutInterface;
use crate::renderer::materials::pbr_material::PhysicalMaterialDescriptor;
use crate::renderer::pipeline_manager::PipelineDefinition;
use std::cell::RefCell;
use std::sync::Arc;

const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub struct PipelineVisitorEnvironment<'a> {
    pub(crate) pipeline_definition: &'a PipelineDefinition,
    pub context: &'a RenderContext<'a>,
    pub layout: Arc<RefCell<LayoutInterface>>,
}

pub struct MaterialFactory<'a> {
    render_context: &'a RenderContext<'a>,
    layout_interface: Arc<RefCell<LayoutInterface>>,
}

impl<'a> MaterialFactory<'a> {
    pub(in crate::renderer) fn new(
        render_context: &'a RenderContext<'a>,
        layout_interface: Arc<RefCell<LayoutInterface>>,
    ) -> Self {
        Self {
            render_context,
            layout_interface,
        }
    }

    pub fn create_material<M: MaterialTrait>(&self, desc: M::Descriptor) -> M {
        let bind_group_layout = Arc::clone(
            self.layout_interface
                .borrow()
                .materials
                .get(&M::TYPE)
                .expect("Layout not found for material type {M::TYPE}"),
        );
        M::create(self.render_context, desc, &bind_group_layout)
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
