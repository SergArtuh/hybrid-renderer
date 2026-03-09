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
        name: String::from("test"),
        scene_roots: Vec::new(),
    };

    for scene in document.scenes() {
        for node in scene.nodes() {
            let root_node = load_node(&node, &buffers, device)?;
            gltf_asset.scene_roots.push(root_node);
        }
    }

    Ok(gltf_asset)
}

fn load_node(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    device: &wgpu::Device,
) -> anyhow::Result<Arc<ModelNode>> {
    let local_matrix = glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    let (mesh, material) = if let Some(gltf_mesh) = node.mesh() {
        let primitive = gltf_mesh.primitives().next().unwrap();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let mesh_data = MeshData {
            vertices: reader.read_positions().unwrap().collect(),
            normals: reader
                .read_normals()
                .map(|i| i.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; 1]),
            uvs: reader
                .read_tex_coords(0)
                .map(|i| i.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; 1]),
            indices: reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_default(),
        };

        let mesh = Arc::new(Mesh::from_data(device, &mesh_data));
        let material = Arc::new(Material::Physical(PhysicalMaterial::default()));
        (Some(mesh), Some(material))
    } else {
        (None, None)
    };

    let mut children = Vec::new();
    for child in node.children() {
        children.push(load_node(&child, buffers, device)?);
    }

    Ok(Arc::new(ModelNode {
        local_transform: local_matrix,
        children,
        mesh,
        material,
    }))
}
