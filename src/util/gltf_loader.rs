use crate::core::material::{Material, PhysicalMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use std::path::Path;
use std::sync::Arc;

pub struct GltfAsset {
    pub name: String,
    //pub nodes: HashMap<String, Arc<ModelNode>>,
    pub scene_roots: Vec<Arc<ModelNode>>,
}

pub fn load_gltf_models(
    path: impl AsRef<Path>,
    device: &wgpu::Device,
) -> anyhow::Result<GltfAsset> {
    let (document, buffers, _) = gltf::import(path)?;

    let mut gltf_asset = GltfAsset {
        name: String::from("test"), // TODO: Use the actual name of the model
        scene_roots: Vec::new(),
    };

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

            gltf_asset
                .scene_roots
                .push(Arc::new(ModelNode::new(mesh, material)));
        }
    }

    Ok(gltf_asset)
}
