use crate::core::material::Material;
use crate::core::mesh::Mesh;

pub struct Model {
    pub mesh: Mesh,
    pub material: Material,
}

impl Model {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self { mesh, material }
    }
}
