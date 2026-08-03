use std::sync::Arc;

use crate::core::model_node::ModelNode;

#[derive(Clone)]
pub struct Model {
    pub root_node: Arc<ModelNode>,
    pub transform: glam::Mat4,
}

impl Model {
    pub fn new(root_node: Arc<ModelNode>, transform: glam::Mat4) -> Self {
        Self {
            root_node,
            transform,
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
