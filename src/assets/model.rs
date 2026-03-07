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
}
