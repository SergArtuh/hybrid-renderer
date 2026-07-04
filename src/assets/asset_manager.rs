use crate::assets::skydome::Skydome;
use crate::assets::util::gltf_extended_decorator::{
    ClearcoatFactor, ClearcoatRoughnessFactor, ExtendedMaterialDecorator,
};
use crate::core::material::{Material, PhysicalMaterial, SkydomeEnvironmentMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use crate::core::texture::Texture;
use crate::core::texture_builder::{ComponentPrecision, TextureBuilder, TextureChannels};
use crate::renderer::RenderingEnvironment;
use crate::renderer::compute_task::{
    DiffuseIrradianceProvider, EquirectToCubemapProvider, MipmapGeneratorProvider,
    SpecularPrefilterProvider,
};
use crate::renderer::materials::pbr_material::PhysicalMaterialDescriptor;
use crate::renderer::materials::{MaterialFactory, SkydomeMaterialDescriptor};
use crate::util::geometry_generator::MeshUtil;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

pub struct GltfAsset {
    pub name: String,
    pub scene_roots: Vec<Arc<ModelNode>>,
}

pub struct AssetManager<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

impl<'a> AssetManager<'a> {
    pub fn new(render_env: &'a RenderingEnvironment) -> Self {
        Self { render_env }
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

        let material_factory = MaterialFactory::new(&self.render_env);
        for scene in document.scenes() {
            for node in scene.nodes() {
                let root_node =
                    self.load_node(&node, &buffers, &textures, &document, &material_factory)?;
                gltf_asset.scene_roots.push(root_node);
            }
        }

        println!("Loaded gltf asset: {}", asset_name);
        Ok(gltf_asset)
    }

    pub fn load_skydome(
        &self,
        path: impl AsRef<Path>,
        radius: f32,
        factor: f32,
    ) -> anyhow::Result<Skydome> {
        let mut file = File::open(path).expect("File not found!");
        let mut diffuse_bytes = Vec::<u8>::new();
        file.read_to_end(&mut diffuse_bytes).unwrap();

        let environment_map_texture = Arc::new(
            TextureBuilder::new(
                &self.render_env.render_context.device,
                &self.render_env.render_context.queue,
            )
            .from_bytes(&diffuse_bytes)
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_filter(wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
            .build(),
        );

        let mesh = Arc::new(Mesh::from_data(
            &self.render_env.render_context.device,
            &MeshUtil::new_procedural_quad(),
        ));

        let material_factory = MaterialFactory::new(&self.render_env);

        let cubemap_texture_lod_0 = EquirectToCubemapProvider::new(&self.render_env)
            .process(&environment_map_texture)
            .unwrap();

        let cubemap_texture = MipmapGeneratorProvider::new(&self.render_env)
            .process(&cubemap_texture_lod_0)
            .unwrap();

        let irradiance_cubemap = DiffuseIrradianceProvider::new(&self.render_env)
            .process(&cubemap_texture, 16)
            .unwrap();

        let specular_cubemap = SpecularPrefilterProvider::new(&self.render_env)
            .process(&cubemap_texture)
            .unwrap();

        let material = material_factory.create_material::<SkydomeEnvironmentMaterial>(
            SkydomeMaterialDescriptor {
                skybox_texture: Arc::clone(&cubemap_texture),
                irradiance_texture: Arc::clone(&irradiance_cubemap),
                specular_texture: Arc::clone(&specular_cubemap),
                dome_radius: radius,
                dome_factor: factor,
            },
        );

        Ok(Skydome {
            mesh,
            material: Arc::new(Material::Skydome(material)),
        })
    }

    fn load_node(
        &self,
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        textures: &Vec<Arc<Texture>>,
        document: &gltf::Document,
        material_factory: &MaterialFactory,
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
                tangents: reader
                    .read_tangents()
                    .map(|i| i.collect())
                    .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; vertex_count]),
                uvs: reader
                    .read_tex_coords(0)
                    .map(|i| i.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; vertex_count]),
                indices: reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_default(),
            };

            let mesh = Arc::new(Mesh::from_data(
                &self.render_env.render_context.device,
                &mesh_data,
            ));

            let gltf_material_base = primitive.material();
            let gltf_material = ExtendedMaterialDecorator::new(gltf_material_base, &document);
            let pbr = gltf_material.pbr_metallic_roughness();

            let mut desc = PhysicalMaterialDescriptor {
                base_color: None,
                normal: None,
                metallic_roughness: None,
                occlusion: None,
                emissive: None,
                base_color_factor: pbr.base_color_factor(),
                emissive_factor: gltf_material.emissive_factor(),
                normal_scale: 1.0,
                roughness_factor: pbr.roughness_factor(),
                metallic_factor: pbr.metallic_factor(),
                occlusion_strength: 1.0,
                clearcoat_factor: 0.0,
                clearcoat_roughness: 0.0,
                clearcoat_texture: None,
                clearcoat_roughness_texture: None,
                clearcoat_normal_texture: None,
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
                desc.normal_scale = texture_info.scale();
            }

            if let Some(texture_info) = gltf_material.occlusion_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.occlusion = Some(Arc::clone(&textures[texture_index]));
                desc.occlusion_strength = texture_info.strength();
            }

            if let Some(texture_info) = gltf_material.emissive_texture() {
                let texture_index = texture_info.texture().source().index();
                desc.emissive = Some(Arc::clone(&textures[texture_index]));
            }

            if let Some(ref cc) = gltf_material.clearcoat {
                let ClearcoatFactor(factor) = cc.clearcoat_factor;
                let ClearcoatRoughnessFactor(roughness) = cc.clearcoat_roughness_factor;

                desc.clearcoat_factor = factor;
                desc.clearcoat_roughness = roughness;

                if let Some(ref tex_info) = cc.clearcoat_texture {
                    let texture_index = tex_info.texture.source().index();
                    desc.clearcoat_texture = Some(Arc::clone(&textures[texture_index]));
                }

                if let Some(ref rough_info) = cc.clearcoat_roughness_texture {
                    let texture_index = rough_info.texture.source().index();
                    desc.clearcoat_roughness_texture = Some(Arc::clone(&textures[texture_index]));
                }

                if let Some(ref normal_info) = cc.clearcoat_normal_texture {
                    let texture_index = normal_info.texture.source().index();
                    desc.clearcoat_normal_texture = Some(Arc::clone(&textures[texture_index]));
                }
            }

            let material = Arc::new(Material::Physical(
                material_factory.create_material::<PhysicalMaterial>(desc),
            ));
            (Some(mesh), Some(material))
        } else {
            (None, None)
        };

        let mut children = Vec::new();
        for child in node.children() {
            children.push(self.load_node(
                &child,
                buffers,
                textures,
                &document,
                material_factory,
            )?);
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

        let mut is_srgb_image = vec![false; images.len()];
        for material in document.materials() {
            if let Some(tex_info) = material.pbr_metallic_roughness().base_color_texture() {
                is_srgb_image[tex_info.texture().source().index()] = true;
            }
            if let Some(tex_info) = material.emissive_texture() {
                is_srgb_image[tex_info.texture().source().index()] = true;
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
                let builder = TextureBuilder::new(
                    &self.render_env.render_context.device,
                    &self.render_env.render_context.queue,
                )
                .from_raw(&img_data.pixels, img_data.width, img_data.height)
                .with_label(label)
                .with_channels(channels)
                .with_srgb(is_srgb_image[i])
                .with_precision(precision);

                Arc::new(builder.build())
            })
            .collect()
    }
}
