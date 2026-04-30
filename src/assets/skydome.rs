use std::sync::Arc;

use crate::core::{material::Material, mesh::Mesh};

#[derive(Clone)]
pub struct Skydome {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
}
