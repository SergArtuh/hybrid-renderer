use crate::core::material::{Material, PhysicalMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model::Model;
use std::path::Path;

pub fn load_gltf_models(
    path: impl AsRef<Path>,
    device: &wgpu::Device,
) -> anyhow::Result<Vec<Model>> {
    let (document, buffers, _) = gltf::import(path)?;
    let mut models = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let vertices: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|i| i.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; vertices.len()]);
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|i| i.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; vertices.len()]);
            let indices: Vec<u32> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_default();

            let mesh_data = MeshData {
                vertices,
                normals,
                uvs,
                indices,
            };

            let mesh = Mesh::from_data(device, &mesh_data);
            let material = Material::Physical(PhysicalMaterial::default());

            models.push(Model::new(mesh, material));
        }
    }

    Ok(models)
}
