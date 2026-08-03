use crate::assets::skydome::Skydome;
use crate::core::material::{EnvironmentMap, Material};
use crate::core::mesh::Mesh;
use crate::core::uniforms::CameraUniform;
use std::sync::Arc;

#[derive(Clone)]
pub enum RenderItem {
    StaticMesh {
        mesh: Arc<Mesh>,
        material: Arc<Material>,
        world_matrix: glam::Mat4,
    },
}

impl RenderItem {
    pub fn world_matrix(&self) -> &glam::Mat4 {
        match self {
            RenderItem::StaticMesh { world_matrix, .. } => world_matrix,
        }
    }
}

pub struct FrameData {
    pub camera_uniform: CameraUniform,
    pub render_items_opaque: Vec<RenderItem>,
    pub render_items_opaque_offset: usize,
    pub render_items_transparent: Vec<RenderItem>,
    pub render_items_transparent_offset: usize,
    pub skydome: Option<Skydome>,
}

impl FrameData {
    pub fn get_environment_texture(&self) -> Option<&EnvironmentMap> {
        match self.skydome.as_ref()?.material.as_ref() {
            Material::Skydome(m) => Some(&m.environment_map),
            _ => None,
        }
    }
}
