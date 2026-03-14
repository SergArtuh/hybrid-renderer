use crate::core::material::Material;
use crate::core::material_builder::PhysicalMaterialBuilder;
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use crate::core::render_context::RenderContext;
use crate::core::texture::Texture;
use crate::core::texture_builder::TextureBuilder;
use crate::renderer::layout_interface::LayoutInterface;
use std::path::Path;
use std::sync::Arc;

pub struct GltfAsset {
    pub name: String,
    //pub nodes: HashMap<String, Arc<ModelNode>>,
    pub scene_roots: Vec<Arc<ModelNode>>,
}

pub struct SceneLoader<'a> {
    pub ctx: &'a RenderContext<'a>,
    pub interface: Arc<LayoutInterface>,
}

impl<'a> SceneLoader<'a> {
    pub fn new(ctx: &'a RenderContext, interface: Arc<LayoutInterface>) -> Self {
        Self { ctx, interface }
    }

    pub fn load_gltf_models(&self, path: impl AsRef<Path>) -> anyhow::Result<GltfAsset> {
        let (document, buffers, images) = gltf::import(path)?;

        let mut gltf_asset = GltfAsset {
            name: String::from("test"),
            scene_roots: Vec::new(),
        };

        let mut image_names = vec![None; images.len()];

        for texture in document.textures() {
            let img_index = texture.source().index();
            if let Some(name) = texture.name() {
                image_names[img_index] = Some(name.to_string());
            }
        }

        let textures: Vec<Arc<Texture>> = images
            .iter()
            .enumerate()
            .map(|(i, img_data)| {
                let label = image_names[i].as_deref().unwrap_or("gltf_texture_unnamed");
                println!("Texture in load_gltf_models label: {}", label);
                println!("Texture in load_gltf_models format: {:?}", img_data.format);
                println!("Texture in load_gltf_models width: {}", img_data.width);
                println!("Texture in load_gltf_models height: {}", img_data.height);
                println!(
                    "Texture in load_gltf_models pixels: {}",
                    img_data.pixels.len()
                );
                println!("\n\n\n");
                Arc::new(
                    TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                        .from_raw(&img_data.pixels, img_data.width, img_data.height)
                        .with_label(label)
                        .with_format(img_data.format, false)
                        .build(),
                )
            })
            .collect();

        for scene in document.scenes() {
            for node in scene.nodes() {
                let root_node = self.load_node(&node, &buffers, &textures)?;
                gltf_asset.scene_roots.push(root_node);
            }
        }

        Ok(gltf_asset)
    }

    fn load_node(
        &self,
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        textures: &Vec<Arc<Texture>>,
    ) -> anyhow::Result<Arc<ModelNode>> {
        let local_matrix = glam::Mat4::from_cols_array_2d(&node.transform().matrix());

        let (mesh, material) = if let Some(gltf_mesh) = node.mesh() {
            let primitive = gltf_mesh.primitives().next().unwrap();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let vertex_count = reader.read_positions().unwrap().count();
            let mesh_data = MeshData {
                vertices: reader.read_positions().unwrap().collect(),
                normals: reader
                    .read_normals()
                    .map(|i| i.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; vertex_count]),
                uvs: reader
                    .read_tex_coords(0)
                    .map(|i| i.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; vertex_count]),
                indices: reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_default(),
            };

            let mesh = Arc::new(Mesh::from_data(&self.ctx.device, &mesh_data));

            let gltf_material = primitive.material();
            let pbr = gltf_material.pbr_metallic_roughness();

            let mut physical_material_builder = PhysicalMaterialBuilder::new();
            if let Some(texture_info) = pbr.base_color_texture() {
                let texture_index = texture_info.texture().source().index();
                let texture = Arc::clone(&textures[texture_index]);

                physical_material_builder = physical_material_builder.with_base_color(texture);
            }

            if let Some(texture_info) = pbr.metallic_roughness_texture() {
                let texture_index = texture_info.texture().source().index();
                let texture = Arc::clone(&textures[texture_index]);

                physical_material_builder =
                    physical_material_builder.with_metallic_roughness(texture);
            }

            if let Some(texture_info) = gltf_material.normal_texture() {
                let texture_index = texture_info.texture().source().index();
                let texture = Arc::clone(&textures[texture_index]);

                physical_material_builder = physical_material_builder.with_normal(texture);
            }

            if let Some(texture_info) = gltf_material.occlusion_texture() {
                let texture_index = texture_info.texture().source().index();
                let texture = Arc::clone(&textures[texture_index]);

                physical_material_builder = physical_material_builder.with_occlusion(texture);
            }

            if let Some(texture_info) = gltf_material.emissive_texture() {
                let texture_index = texture_info.texture().source().index();
                let texture = Arc::clone(&textures[texture_index]);

                physical_material_builder = physical_material_builder.with_emissive(texture);
            }

            let material = Arc::new(Material::Physical(
                physical_material_builder.build(&self.ctx, &self.interface.material),
            ));
            (Some(mesh), Some(material))
        } else {
            (None, None)
        };

        let mut children = Vec::new();
        for child in node.children() {
            children.push(self.load_node(&child, buffers, textures)?);
        }

        Ok(Arc::new(ModelNode {
            local_transform: local_matrix,
            children,
            mesh,
            material,
        }))
    }
}
