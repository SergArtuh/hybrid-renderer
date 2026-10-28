use std::sync::Arc;

use crate::core::{model_node::ModelNode, sdf_volume::SdfVolume};

pub struct ModelAsset {
    pub name: String,
    pub scene_roots: Vec<Arc<ModelNode>>,
}

#[derive(Clone)]
pub struct Model {
    pub name: String,
    pub root_node: Arc<ModelNode>,
    pub transform: glam::Mat4,
    pub sdf_volumes: Vec<SdfVolume>,
}

impl Model {
    pub fn new(
        root_node: Arc<ModelNode>,
        transform: glam::Mat4,
        sdf_volumes: Vec<SdfVolume>,
    ) -> Self {
        Self {
            name: "Default Model Name".to_string(),
            root_node,
            transform,
            sdf_volumes,
        }
    }

    pub fn find_node_by_name(&self, name: &str) -> Option<Arc<ModelNode>> {
        Self::find_node_by_name_recursive(name, &self.root_node)
    }

    fn find_node_by_name_recursive(name: &str, node: &Arc<ModelNode>) -> Option<Arc<ModelNode>> {
        if node.name == name {
            return Some(Arc::clone(&node));
        }

        for child in &node.children {
            if let Some(node) = Self::find_node_by_name_recursive(name, child) {
                return Some(node);
            }
        }

        None
    }
}
