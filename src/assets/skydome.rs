use std::sync::Arc;

use crate::core::{material::Material, mesh::Mesh};

#[derive(Clone)]
pub struct SkydomeDescriptor {
    pub radius: f32,
    pub factor: f32,
}

#[derive(Clone)]
pub struct Skydome {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
}
