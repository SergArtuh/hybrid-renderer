use crate::core::material::{Material, PhysicalMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use crate::core::render_context::RenderContext;
use crate::core::texture::Texture;
use crate::core::texture_builder::{ComponentPrecision, TextureBuilder, TextureChannels};
use crate::renderer::materials::MaterialFactory;
use crate::renderer::materials::pbr_material::PhysicalMaterialDescriptor;
use std::path::Path;
use std::sync::Arc;

pub struct GltfAsset {
    pub name: String,
    //pub nodes: HashMap<String, Arc<ModelNode>>,
    pub scene_roots: Vec<Arc<ModelNode>>,
}

pub struct AssetLoader<'ctx, 'fac> {
    pub ctx: &'ctx RenderContext<'ctx>,
    pub material_factory: &'fac MaterialFactory<'ctx>,
}

impl<'ctx, 'fac> AssetLoader<'ctx, 'fac> {
    pub fn new(
        ctx: &'ctx RenderContext<'ctx>,
        material_factory: &'fac MaterialFactory<'ctx>,
    ) -> Self {
        Self {
            ctx,
            material_factory,
        }
    }

    pub fn load_gltf_models(&self, path: impl AsRef<Path>) -> anyhow::Result<GltfAsset> {
        let (document, buffers, images) = gltf::import(path)?;

        let asset_name = document
            .default_scene()
            .and_then(|s| s.name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled_Scene".to_string());

        let mut gltf_asset = GltfAsset {
            name: asset_name.clone(),
            scene_roots: Vec::new(),
        };

        let textures = self.load_textures(&document, &images);

        for scene in document.scenes() {
            for node in scene.nodes() {
                let root_node = self.load_node(&node, &buffers, &textures)?;
                gltf_asset.scene_roots.push(root_node);
            }
        }

        println!("Loaded gltf asset: {}", asset_name);
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

            let mut desc = PhysicalMaterialDescriptor {
                base_color: None,
                normal: None,
                metallic_roughness: None,
                occlusion: None,
                emissive: None,
            };

            if let Some(texture_info) = pbr.base_color_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.base_color = Some(Arc::clone(&textures[texture_index]));
            }

            if let Some(texture_info) = pbr.metallic_roughness_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.metallic_roughness = Some(Arc::clone(&textures[texture_index]));
            }

            if let Some(texture_info) = gltf_material.normal_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.normal = Some(Arc::clone(&textures[texture_index]));
            }

            if let Some(texture_info) = gltf_material.occlusion_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.occlusion = Some(Arc::clone(&textures[texture_index]));
            }

            if let Some(texture_info) = gltf_material.emissive_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.emissive = Some(Arc::clone(&textures[texture_index]));
            }

            let material = Arc::new(Material::Physical(
                self.material_factory
                    .create_material::<PhysicalMaterial>(desc),
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

    fn load_textures(
        &self,
        document: &gltf::Document,
        images: &[gltf::image::Data],
    ) -> Vec<Arc<Texture>> {
        let mut image_names = vec![None; images.len()];

        for texture in document.textures() {
            let img_index = texture.source().index();
            if let Some(name) = texture.name() {
                image_names[img_index] = Some(name.to_string());
            }
        }

        images
            .iter()
            .enumerate()
            .map(|(i, img_data)| {
                let label = image_names[i].as_deref().unwrap_or("gltf_texture_unnamed");
                let channels = match img_data.format {
                    gltf::image::Format::R8 => TextureChannels::R,
                    gltf::image::Format::R8G8 => TextureChannels::RG,
                    gltf::image::Format::R8G8B8 => TextureChannels::RGB,
                    gltf::image::Format::R8G8B8A8 => TextureChannels::RGBA,
                    _ => panic!("Unsupported format: {:?}", img_data.format),
                };

                let precision = match img_data.format {
                    gltf::image::Format::R8 => ComponentPrecision::U8,
                    gltf::image::Format::R8G8 => ComponentPrecision::U8,
                    gltf::image::Format::R8G8B8 => ComponentPrecision::U8,
                    gltf::image::Format::R8G8B8A8 => ComponentPrecision::U8,
                    _ => panic!("Unsupported format: {:?}", img_data.format),
                };

                Arc::new(
                    TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                        .from_raw(&img_data.pixels, img_data.width, img_data.height)
                        .with_label(label)
                        .with_channels(channels)
                        .with_precision(precision)
                        .build(),
                )
            })
            .collect()
    }
}
