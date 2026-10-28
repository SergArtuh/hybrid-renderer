use crate::assets::assets_resources::{
    AssetCallback, AssetData, AssetResource, CompletedAssetTask, ModelAssetData,
    SdfVolumeAssetData, SkydomeAssetData, TextureAssetData,
};
use crate::assets::model::ModelAsset;
use crate::assets::skydome::{Skydome, SkydomeDescriptor};
use crate::assets::util::gltf_extended_decorator::{
    ClearcoatFactor, ClearcoatRoughnessFactor, ExtendedMaterialDecorator,
};
use crate::core::material::{AlphaMode, Material, PhysicalMaterial, SkydomeEnvironmentMaterial};
use crate::core::mesh::{Mesh, MeshData};
use crate::core::model_node::ModelNode;
use crate::core::sdf_volume::SdfVolume;
use crate::core::texture::Texture;
use crate::core::texture_builder::{
    ComponentPrecision, TextureBuilder, TextureChannels, TextureDescriptor, TextureFormatDescriptor,
};
use crate::core::uniforms::SDFUniform;
use crate::renderer::RenderingEnvironment;
use crate::renderer::compute_task::{
    DiffuseIrradianceProvider, EquirectToCubemapProvider, MipmapGeneratorProvider,
    SpecularPrefilterProvider,
};
use crate::renderer::materials::PhysicalMaterialDescriptor;
use crate::renderer::materials::{MaterialFactory, SkydomeMaterialDescriptor};
use crate::stage::Stage;
use crate::util::geometry_generator::MeshUtil;
use gltf::material::AlphaMode as GltfAlphaMode;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::path::Path;
use std::sync::Arc;

use crate::core::uniforms::RawSdfConfig;
use futures_util::future::try_join;

use uuid::Uuid;

use std::sync::mpsc::Sender;

pub type SharedAssetLoadingGroup = Arc<RefCell<AssetLoadingGroup>>;

pub enum GroupExecutionMode {
    Parallel,
    Sequential,
}

pub struct AssetLoadingGroup {
    mode: GroupExecutionMode,
    callbacks_uids: Vec<Uuid>,
    callbacks_ready: HashSet<Uuid>,
}

impl AssetLoadingGroup {
    pub fn create_parallel(render_env: &mut RenderingEnvironment) -> SharedAssetLoadingGroup {
        let group = Arc::new(RefCell::new(AssetLoadingGroup {
            mode: GroupExecutionMode::Parallel,
            callbacks_uids: Vec::new(),
            callbacks_ready: HashSet::new(),
        }));

        render_env
            .asset_resources
            .loading_groups
            .push(Arc::clone(&group));

        group
    }

    #[allow(dead_code)]
    fn create_sequential_group() -> SharedAssetLoadingGroup {
        todo!()
    }

    pub fn register_task(&mut self, callback_uid: Uuid) {
        self.callbacks_uids.push(callback_uid);
    }

    pub fn contains_task(&self, callback_uid: &Uuid) -> bool {
        self.callbacks_uids.contains(callback_uid)
    }

    pub fn mark_task_ready(&mut self, callback_uid: &Uuid) {
        self.callbacks_ready.insert(*callback_uid);
    }

    pub fn get_tasks_to_execute(&mut self) -> Vec<Uuid> {
        match self.mode {
            GroupExecutionMode::Parallel => {
                if self.is_ready() {
                    mem::take(&mut self.callbacks_uids)
                } else {
                    Vec::new()
                }
            }
            GroupExecutionMode::Sequential => todo!(),
        }
    }

    pub fn is_disposed(&self) -> bool {
        self.callbacks_uids.is_empty()
    }

    fn is_ready(&self) -> bool {
        match self.mode {
            GroupExecutionMode::Parallel => {
                !self.callbacks_uids.is_empty()
                    && self
                        .callbacks_uids
                        .iter()
                        .all(|uid| self.callbacks_ready.contains(uid))
            }
            GroupExecutionMode::Sequential => todo!(),
        }
    }
}

pub struct AssetManager<'a> {
    pub render_env: &'a RenderingEnvironment<'a>,
    sender: Sender<CompletedAssetTask>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> AssetManager<'a> {
    pub fn new(render_env: &'a RenderingEnvironment) -> Self {
        Self {
            render_env,
            sender: render_env.asset_resources.sender.clone(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update(render_env: &mut RenderingEnvironment, stage: &mut Stage) {
        let tasks = Self::process_loaded_asset(&mut render_env.asset_resources);

        //let pending_tasks = std::mem::take(&mut render_env.asset_resources.pending_tasks);
        for task in tasks {
            if let AssetData::Texture(texture_data) = &task.data {
                let texture = TextureBuilder::new(
                    &render_env.render_context.device,
                    &render_env.render_context.queue,
                )
                .from_bytes(&texture_data.bytes)
                .with_format(texture_data.descriptor.format)
                // TODO: check descriptor for filter mode
                .with_filter(wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
                .build();

                if let AssetCallback::Texture(callback) = task.callback {
                    callback(render_env, stage, texture);
                }
            } else if let AssetData::Skydome(skydome_data) = task.data {
                let skydome =
                    Self::create_skydome(render_env, skydome_data.bytes, skydome_data.descriptor);

                if let AssetCallback::Skydome(callback) = task.callback {
                    callback(render_env, stage, skydome);
                }
            } else if let AssetData::Model(model_data) = task.data {
                let model = Self::create_model(render_env, model_data.bytes).unwrap();
                if let AssetCallback::Model(callback) = task.callback {
                    callback(render_env, stage, model);
                }
            } else if let AssetData::SdfVolume(sdf_volume_data) = task.data {
                let texture = TextureBuilder::new(
                    &render_env.render_context.device,
                    &render_env.render_context.queue,
                )
                .from_bytes(&sdf_volume_data.texture_data.bytes)
                .with_format(sdf_volume_data.texture_data.descriptor.format)
                //.with_filter(wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
                .build();

                let sdf_uniform = Self::make_sdf_uniform(sdf_volume_data.metadata_bytes).unwrap();

                let sdf_volume = SdfVolume {
                    world_to_sdf_matrix: glam::Mat4::IDENTITY.inverse(),
                    texture_view: texture.view.as_ref().clone(),
                    sdf_uniform,
                };
                if let AssetCallback::SdfVolume(callback) = task.callback {
                    callback(render_env, stage, sdf_volume);
                }
            } else if let AssetData::Bytes(bytes) = task.data
                && let AssetCallback::Bytes(callback) = task.callback
            {
                callback(render_env, stage, bytes);
            }
        }
    }

    pub fn load_texture<F>(
        &self,
        path_or_url: impl AsRef<Path> + Send + 'static,
        descriptor: TextureDescriptor,
        on_loaded: F,
        loading_group: Option<SharedAssetLoadingGroup>,
    ) where
        F: FnOnce(&mut RenderingEnvironment, &mut Stage, Texture) + Send + 'static,
    {
        let path = path_or_url.as_ref().to_string_lossy().to_string();
        let sender = self.sender.clone();
        let callback = Box::new(on_loaded);

        let uid = Uuid::new_v4();

        if let Some(group) = &loading_group {
            group.as_ref().borrow_mut().register_task(uid);
        }

        Self::spawn_task(async move {
            if let Ok(bytes) = Self::fetch_bytes(&path).await {
                let _ = sender.send(CompletedAssetTask {
                    data: AssetData::Texture(TextureAssetData { bytes, descriptor }),
                    callback: AssetCallback::Texture(callback),
                    uid,
                });
            } else {
                #[cfg(debug_assertions)]
                eprintln!("[AssetManager] Failed to load texture: {}", path);
            }
        });
    }

    pub fn load_sdf_from_texture<F>(
        &self,
        texture_path_or_url: impl AsRef<Path> + Send + 'static,
        metadata_path_or_url: impl AsRef<Path> + Send + 'static,
        on_loaded: F,
        loading_group: Option<SharedAssetLoadingGroup>,
    ) where
        F: FnOnce(&mut RenderingEnvironment, &mut Stage, SdfVolume) + Send + 'static,
    {
        let texture_path = texture_path_or_url.as_ref().to_string_lossy().to_string();
        let metadata_path = metadata_path_or_url.as_ref().to_string_lossy().to_string();
        let sender = self.sender.clone();
        let callback = Box::new(on_loaded);

        let descriptor = TextureDescriptor {
            format: TextureFormatDescriptor {
                channels: TextureChannels::RGBA,
                precision: ComponentPrecision::F16,
                is_srgb: false,
            },
            ..Default::default()
        };

        let uid = Uuid::new_v4();

        if let Some(group) = &loading_group {
            group.as_ref().borrow_mut().register_task(uid);
        }

        Self::spawn_task(async move {
            let fetched_data = try_join(
                Self::fetch_bytes(&texture_path),
                Self::fetch_bytes(&metadata_path),
            )
            .await;

            if let Ok((bytes_texture, bytes_metadata)) = fetched_data {
                let _ = sender.send(CompletedAssetTask {
                    data: AssetData::SdfVolume(SdfVolumeAssetData {
                        metadata_bytes: bytes_metadata,
                        texture_data: TextureAssetData {
                            bytes: bytes_texture,
                            descriptor,
                        },
                    }),
                    callback: AssetCallback::SdfVolume(callback),
                    uid,
                });
            } else {
                #[cfg(debug_assertions)]
                eprintln!("[AssetManager] Failed to load texture: {}", texture_path);
            }
        });
    }

    pub fn load_skydome<F>(
        &self,
        path_or_url: impl AsRef<Path> + Send + 'static,
        descriptor: SkydomeDescriptor,
        on_loaded: F,
        loading_group: Option<SharedAssetLoadingGroup>,
    ) where
        F: FnOnce(&mut RenderingEnvironment, &mut Stage, Skydome) + Send + 'static,
    {
        let path = path_or_url.as_ref().to_string_lossy().to_string();
        let sender = self.sender.clone();
        let callback = Box::new(on_loaded);

        let uid = Uuid::new_v4();

        if let Some(group) = &loading_group {
            group.as_ref().borrow_mut().register_task(uid);
        }

        Self::spawn_task(async move {
            if let Ok(bytes) = Self::fetch_bytes(&path).await {
                let _ = sender.send(CompletedAssetTask {
                    data: AssetData::Skydome(SkydomeAssetData { bytes, descriptor }),
                    callback: AssetCallback::Skydome(callback),
                    uid,
                });
            } else {
                #[cfg(debug_assertions)]
                eprintln!("[AssetManager] Failed to load skydome: {}", path);
            }
        });
    }

    pub fn create_skydome(
        render_env: &RenderingEnvironment,
        bytes: Vec<u8>,
        descriptor: SkydomeDescriptor,
    ) -> Skydome {
        let environment_map_texture = Arc::new(
            TextureBuilder::new(
                &render_env.render_context.device,
                &render_env.render_context.queue,
            )
            .from_bytes(&bytes)
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_filter(wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
            .build(),
        );

        let mesh = Arc::new(Mesh::from_data(
            &render_env.render_context.device,
            &MeshUtil::new_procedural_quad(),
        ));

        let material_factory = MaterialFactory::new(&render_env);

        let cubemap_texture_lod_0 = EquirectToCubemapProvider::new(&render_env)
            .process(&environment_map_texture)
            .unwrap();

        let cubemap_texture = MipmapGeneratorProvider::new(&render_env)
            .process(&cubemap_texture_lod_0)
            .unwrap();

        let irradiance_cubemap = DiffuseIrradianceProvider::new(&render_env)
            .process(&cubemap_texture, 16)
            .unwrap();

        let specular_cubemap = SpecularPrefilterProvider::new(&render_env)
            .process(&cubemap_texture)
            .unwrap();

        let material = material_factory.create_material::<SkydomeEnvironmentMaterial>(
            SkydomeMaterialDescriptor {
                skybox_texture: Arc::clone(&cubemap_texture),
                irradiance_texture: Arc::clone(&irradiance_cubemap),
                specular_texture: Arc::clone(&specular_cubemap),
                dome_radius: descriptor.radius,
                dome_factor: descriptor.factor,
            },
        );

        Skydome {
            mesh,
            material: Arc::new(Material::Skydome(material)),
        }
    }

    pub fn load_model<F>(
        &self,
        path_or_url: impl AsRef<Path> + Send + 'static,
        on_loaded: F,
        loading_group: Option<SharedAssetLoadingGroup>,
    ) where
        F: FnOnce(&mut RenderingEnvironment, &mut Stage, ModelAsset) + Send + 'static,
    {
        let path = path_or_url.as_ref().to_string_lossy().to_string();
        let sender = self.sender.clone();
        let callback = Box::new(on_loaded);

        let uid = Uuid::new_v4();

        if let Some(group) = loading_group {
            group.as_ref().borrow_mut().register_task(uid);
        }

        Self::spawn_task(async move {
            if let Ok(bytes) = Self::fetch_bytes(&path).await {
                let _ = sender.send(CompletedAssetTask {
                    data: AssetData::Model(ModelAssetData { bytes }),
                    callback: AssetCallback::Model(callback),
                    uid,
                });
            } else {
                #[cfg(debug_assertions)]
                eprintln!("[AssetManager] Failed to load model: {}", path);
            }
        });
    }

    pub fn create_model(
        render_env: &RenderingEnvironment,
        bytes: Vec<u8>,
    ) -> anyhow::Result<ModelAsset> {
        let (document, buffers, images) = gltf::import_slice(bytes)?;

        let asset_name = document
            .default_scene()
            .and_then(|s| s.name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled_Scene".to_string());

        let mut gltf_asset = ModelAsset {
            name: asset_name.clone(),
            scene_roots: Vec::new(),
        };

        let textures = Self::load_textures(render_env, &document, &images);

        let material_factory = MaterialFactory::new(&render_env);
        for scene in document.scenes() {
            for node in scene.nodes() {
                let root_node = Self::load_node(
                    render_env,
                    &node,
                    &buffers,
                    &textures,
                    &document,
                    &material_factory,
                )?;
                gltf_asset.scene_roots.push(root_node);
            }
        }

        println!("Loaded gltf asset: {}", asset_name);
        Ok(gltf_asset)
    }

    fn process_loaded_asset(asset_resource: &mut AssetResource) -> Vec<CompletedAssetTask> {
        let tasks: Vec<CompletedAssetTask> = asset_resource.receiver.try_iter().collect();
        asset_resource.pending_tasks.extend(tasks);

        let mut tasks_to_execute_uids: Vec<Uuid> = Vec::new();
        let mut tasks_out_of_groups: Vec<Uuid> = Vec::new();
        asset_resource
            .pending_tasks
            .iter()
            .for_each(|task| tasks_out_of_groups.push(task.uid));

        for group_arc in asset_resource.loading_groups.iter() {
            let mut group = group_arc.as_ref().borrow_mut();
            for task in asset_resource.pending_tasks.iter() {
                if group.contains_task(&task.uid) {
                    group.mark_task_ready(&task.uid);
                    tasks_to_execute_uids.extend(group.get_tasks_to_execute());
                    tasks_out_of_groups.retain(|uid| *uid != task.uid);
                }
            }
        }

        let mut pending_map: HashMap<Uuid, CompletedAssetTask> =
            std::mem::take(&mut asset_resource.pending_tasks)
                .into_iter()
                .map(|task| (task.uid, task))
                .collect();

        tasks_to_execute_uids.extend(tasks_out_of_groups);

        let mut result_tasks: Vec<CompletedAssetTask> = Vec::new();
        for uid in &tasks_to_execute_uids {
            if let Some(task) = pending_map.remove(uid) {
                result_tasks.push(task);
            }
        }

        asset_resource.pending_tasks = pending_map.into_values().collect();
        asset_resource
            .loading_groups
            .retain(|g| !g.borrow().is_disposed());
        result_tasks
    }

    fn load_node(
        render_env: &RenderingEnvironment,
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        textures: &Vec<Arc<Texture>>,
        document: &gltf::Document,
        material_factory: &MaterialFactory,
    ) -> anyhow::Result<Arc<ModelNode>> {
        let local_matrix = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
        let node_name = node.name().unwrap_or("unnamed").to_string();

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
                &render_env.render_context.device,
                &mesh_data,
            ));

            let gltf_material_base = primitive.material();
            let gltf_material = ExtendedMaterialDecorator::new(gltf_material_base, &document);
            let pbr = gltf_material.pbr_metallic_roughness();
            let alpha_mode = match gltf_material.alpha_mode() {
                GltfAlphaMode::Opaque => AlphaMode::Opaque,
                GltfAlphaMode::Mask => AlphaMode::Mask,
                GltfAlphaMode::Blend => AlphaMode::Blend,
            };

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
                alpha_mode,
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
            children.push(Self::load_node(
                render_env,
                &child,
                buffers,
                textures,
                &document,
                material_factory,
            )?);
        }

        Ok(Arc::new(ModelNode {
            is_visible: Cell::new(true),
            name: node_name,
            local_transform: local_matrix,
            children,
            mesh,
            material,
        }))
    }

    fn load_textures(
        render_env: &RenderingEnvironment,
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
                    &render_env.render_context.device,
                    &render_env.render_context.queue,
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

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_task<F>(future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        std::thread::spawn(move || {
            pollster::block_on(future);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_task<F>(future: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future);
    }

    async fn fetch_bytes(path: &str) -> Result<Vec<u8>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;
            use web_sys::{RequestInit, RequestMode, Response, js_sys};

            let opts = RequestInit::new();
            opts.set_method("GET");
            opts.set_mode(RequestMode::Cors);

            let window = web_sys::window().ok_or("No global window found")?;
            let request = web_sys::Request::new_with_str_and_init(path, &opts)
                .map_err(|e| format!("Failed to create request: {:?}", e))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| format!("Fetch failed: {:?}", e))?;

            let resp: Response = resp_value
                .dyn_into()
                .map_err(|_| "Response is not Response type")?;

            if !resp.ok() {
                return Err(format!("HTTP error, status: {}", resp.status()));
            }

            let array_buffer = JsFuture::from(resp.array_buffer().map_err(|e| format!("{:?}", e))?)
                .await
                .map_err(|e| format!("ArrayBuffer failed: {:?}", e))?;

            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            Ok(uint8_array.to_vec())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if path.starts_with("http://") || path.starts_with("https://") {
                let response = ureq::get(path)
                    .call()
                    .map_err(|e| format!("HTTP request failed: {}", e))?;

                let mut bytes = Vec::new();
                response
                    .into_reader()
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("Failed to read response body: {}", e))?;

                Ok(bytes)
            } else {
                std::fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
            }
        }
    }

    pub fn make_sdf_uniform(bytes: Vec<u8>) -> Result<SDFUniform, String> {
        let json_string =
            String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 sequence: {}", e))?;

        let config: RawSdfConfig = serde_json::from_str(&json_string)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let inv_world = glam::Mat4::from_cols_array_2d(&config.sdf_inv_world_matrix);

        Ok(SDFUniform::new(inv_world, config.grid_resolution))
    }
}
