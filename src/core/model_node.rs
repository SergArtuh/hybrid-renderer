use std::sync::Arc;

use crate::core::material::Material;
use crate::core::mesh::Mesh;

pub struct ModelNode {
    pub local_transform: glam::Mat4,
    pub children: Vec<Arc<ModelNode>>,
    pub mesh: Option<Arc<Mesh>>,
    pub material: Option<Arc<Material>>,
}

impl ModelNode {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self {
            local_transform: glam::Mat4::IDENTITY,
            children: Vec::new(),
            mesh: Some(Arc::new(mesh)),
            material: Some(Arc::new(material)),
        }
    }

    pub fn add_child(&mut self, child: ModelNode) {
        self.children.push(Arc::new(child));
    }
}
