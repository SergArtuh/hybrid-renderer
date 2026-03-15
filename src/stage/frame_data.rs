use crate::core::camera::CameraUniform;
use crate::core::material::Material;
use crate::core::mesh::Mesh;
use std::sync::Arc;

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
    pub render_items: Vec<RenderItem>,
}
