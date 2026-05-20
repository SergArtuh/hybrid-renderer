use wgpu::util::DeviceExt;

use crate::assets::skydome::Skydome;
use crate::core::material::{Material, PhysicalMaterial, SkydomeEnvironmentMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use crate::core::render_context::RenderContext;
use crate::core::texture::Texture;
use crate::core::texture_builder::{ComponentPrecision, TextureBuilder, TextureChannels};
use crate::core::uniforms::SpecularPrefilterUniform;
use crate::renderer::compute_task::{
    ComputeTaskFactory, DiffuseIrradianceTask, DiffuseIrradianceTaskDescriptor,
    EquirectToCubemapTask, EquirectToCubemapTaskDescriptor, MipmapGeneratorTask,
    MipmapGeneratorTaskDescriptor, SpecularPrefilterTask, SpecularPrefilterTaskDescriptor,
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

pub struct AssetManager<'ctx> {
    pub ctx: &'ctx RenderContext<'ctx>,
    pub material_factory: MaterialFactory<'ctx>,
    pub compute_task_factory: ComputeTaskFactory<'ctx>,
}

impl<'ctx> AssetManager<'ctx> {
    pub fn new(
        ctx: &'ctx RenderContext<'ctx>,
        material_factory: MaterialFactory<'ctx>,
        compute_task_factory: ComputeTaskFactory<'ctx>,
    ) -> Self {
        Self {
            ctx,
            material_factory,
            compute_task_factory,
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

    pub fn load_skydome(
        &self,
        path: impl AsRef<Path>,
        radius: f32,
        factor: f32,
    ) -> anyhow::Result<Skydome> {
        let mut file = File::open(path).expect("File not found!");
        let mut diffuse_bytes = Vec::<u8>::new();
        file.read_to_end(&mut diffuse_bytes).unwrap();

        let sprite_texture = Arc::new(
            TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                .from_bytes(&diffuse_bytes)
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_filter(wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
                .build(),
        );

        let mesh = Arc::new(Mesh::from_data(
            &self.ctx.device,
            &MeshUtil::new_procedural_quad(),
        ));

        let cubemap_texture_result = Arc::new(
            TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                .with_label("equirect_cubemap")
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(1024, 1024)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                )
                .build(),
        );

        let irradiance_cubemap = Arc::new(
            TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                .with_label("diffuse_irradiance")
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(16, 16)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                )
                .build(),
        );

        let equirect_to_cubemap_task = self
            .compute_task_factory
            .create_task::<EquirectToCubemapTask>(EquirectToCubemapTaskDescriptor {
                input_texture: Arc::clone(&sprite_texture),
                output_cubemap: Arc::clone(&cubemap_texture_result),
            });

        self.compute_task_factory
            .create_executor()
            .record(&self.ctx, &equirect_to_cubemap_task)
            .execute(&self.ctx)
            .wait(&self.ctx);

        let cubemap_texture = self.generate_mipmaps(&cubemap_texture_result).unwrap();

        let diffuse_task = self
            .compute_task_factory
            .create_task::<DiffuseIrradianceTask>(DiffuseIrradianceTaskDescriptor {
                input_cubemap: Arc::clone(&cubemap_texture),
                output_cubemap: Arc::clone(&irradiance_cubemap),
            });

        self.compute_task_factory
            .create_executor()
            .record(&self.ctx, &diffuse_task)
            .execute(&self.ctx)
            .wait(&self.ctx);

        let specular_cubemap = self
            .prefilter_specular_cubemap(&self.ctx, Arc::clone(&cubemap_texture))
            .unwrap();

        let material = self
            .material_factory
            .create_material::<SkydomeEnvironmentMaterial>(SkydomeMaterialDescriptor {
                skybox_texture: Arc::clone(&cubemap_texture),
                irradiance_texture: Arc::clone(&irradiance_cubemap),
                specular_texture: Arc::clone(&specular_cubemap),
                dome_radius: radius,
                dome_factor: factor,
            });

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

    fn generate_mipmaps(
        &self,
        source_texture: &Arc<Texture>,
    ) -> Result<Arc<Texture>, anyhow::Error> {
        let origin_width = source_texture.width;
        let origin_height = source_texture.height;

        let mip_count = (origin_width.max(origin_height) as f32).log2().floor() as u32 + 1;

        let result_texture = Arc::new(
            TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                .with_label("final_mipmapped_cubemap")
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(origin_width, origin_height)
                .with_mip_level_count(mip_count)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                )
                .build(),
        );

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Initial Mip Copy"),
            });
        encoder.copy_texture_to_texture(
            source_texture.texture.as_image_copy(),
            result_texture.texture.as_image_copy(),
            wgpu::Extent3d {
                width: origin_width,
                height: origin_height,
                depth_or_array_layers: 6,
            },
        );
        self.ctx.queue.submit(Some(encoder.finish()));

        let mut current_src = Arc::clone(source_texture);

        for target_mip in 1..mip_count {
            let mip_width = (origin_width >> target_mip).max(1);
            let mip_height = (origin_height >> target_mip).max(1);

            println!("Generating mip {} {}x{}", target_mip, mip_width, mip_height);

            let temp_mip_out = Arc::new(
                TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                    .with_label(&format!("temp_mip_{}", target_mip))
                    .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                    .with_size(mip_width, mip_height)
                    .as_cubemap()
                    .with_usage(
                        wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::STORAGE_BINDING
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::COPY_SRC,
                    )
                    .build(),
            );

            let mipmap_task = self
                .compute_task_factory
                .create_task::<MipmapGeneratorTask>(MipmapGeneratorTaskDescriptor {
                    source_texture: Arc::clone(&current_src),
                    output_texture: Arc::clone(&temp_mip_out),
                });

            self.compute_task_factory
                .create_executor()
                .record(&self.ctx, &mipmap_task)
                .execute(&self.ctx)
                .wait(&self.ctx);

            let mut copy_encoder =
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some(&format!("Copy Mip {}", target_mip)),
                    });

            copy_encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &temp_mip_out.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &result_texture.texture,
                    mip_level: target_mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 6,
                },
            );

            self.ctx.queue.submit(Some(copy_encoder.finish()));
            current_src = temp_mip_out;
        }

        self.ctx.device.poll(wgpu::Maintain::Wait);
        Ok(result_texture)
    }

    pub fn prefilter_specular_cubemap(
        &self,
        render_context: &RenderContext,
        source_cubemap: Arc<Texture>,
    ) -> Result<Arc<Texture>, anyhow::Error> {
        let origin_width = source_cubemap.width;
        let origin_height = source_cubemap.height;

        let mip_count = (origin_width.max(origin_height) as f32).log2().floor() as u32 + 1;

        let result_texture = Arc::new(
            TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                .with_label("final_mipmapped_cubemap")
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(origin_width, origin_height)
                .with_mip_level_count(mip_count)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                )
                .build(),
        );

        let mut executor = self.compute_task_factory.create_executor();

        for mip in 0..mip_count {
            let mip_width = (origin_width >> mip).max(1);
            let mip_height = (origin_height >> mip).max(1);
            let linear_roughness = mip as f32 / (mip_count - 1).max(1) as f32;
            let roughness = linear_roughness * linear_roughness;

            let temp_specular_mip_out = Arc::new(
                TextureBuilder::new(&self.ctx.device, &self.ctx.queue)
                    .with_label(&format!("temp_specular_mip_{}", mip))
                    .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                    .with_size(mip_width, mip_height)
                    .as_cubemap()
                    .with_usage(
                        wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::STORAGE_BINDING
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::COPY_SRC,
                    )
                    .build(),
            );

            let specular_mipmap_data = SpecularPrefilterUniform::new(roughness);
            let specular_mipmap_buffer = Arc::new(render_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Specular Prefilter Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[specular_mipmap_data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            ));

            let prefilter_task = self
                .compute_task_factory
                .create_task::<SpecularPrefilterTask>(SpecularPrefilterTaskDescriptor {
                    input_cubemap: Arc::clone(&source_cubemap),
                    output_cubemap: Arc::clone(&temp_specular_mip_out),
                    config: Arc::clone(&specular_mipmap_buffer),
                });

            executor.record(&self.ctx, &prefilter_task);

            executor
                .get_encoder_mut()
                .expect("Encoder not found")
                .copy_texture_to_texture(
                    wgpu::ImageCopyTexture {
                        texture: &temp_specular_mip_out.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyTexture {
                        texture: &result_texture.texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: mip_width,
                        height: mip_height,
                        depth_or_array_layers: 6,
                    },
                );
        }
        executor.execute(&self.ctx);
        executor.wait(&self.ctx);
        Ok(result_texture)
    }
}
