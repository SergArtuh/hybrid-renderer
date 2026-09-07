use std::sync::{
    Arc,
    mpsc::{Receiver, Sender, channel},
};

use uuid::Uuid;

use crate::{
    assets::{
        asset_manager::AssetLoadingGroup,
        model::ModelAsset,
        skydome::{Skydome, SkydomeDescriptor},
    },
    core::{texture::Texture, texture_builder::TextureDescriptor},
    renderer::RenderingEnvironment,
    stage::Stage,
};

#[derive(Clone)]
pub struct TextureAssetData {
    pub bytes: Vec<u8>,
    pub descriptor: TextureDescriptor,
}

#[derive(Clone)]
pub struct SkydomeAssetData {
    pub bytes: Vec<u8>,
    pub descriptor: SkydomeDescriptor,
}

#[derive(Clone)]
pub struct ModelAssetData {
    pub bytes: Vec<u8>,
}

pub enum AssetData {
    Bytes(Vec<u8>),
    Texture(TextureAssetData),
    Skydome(SkydomeAssetData),
    Model(ModelAssetData),
}

pub type AssetTextureCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Texture) + Send + 'static>;

pub type AssetSkydomeCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Skydome) + Send + 'static>;

pub type AssetModelCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, ModelAsset) + Send + 'static>;

pub type AssetBytesCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Vec<u8>) + Send + 'static>;

pub enum AssetCallback {
    Texture(AssetTextureCallback),
    Skydome(AssetSkydomeCallback),
    Model(AssetModelCallback),
    Bytes(AssetBytesCallback),
}

pub struct CompletedAssetTask {
    pub uid: Uuid,
    pub data: AssetData,
    pub callback: AssetCallback,
}

pub struct AssetResource {
    pub(crate) sender: Sender<CompletedAssetTask>,
    pub(crate) receiver: Receiver<CompletedAssetTask>,
    pub(crate) pending_tasks: Vec<CompletedAssetTask>,
    pub(crate) loading_groups: Vec<Arc<std::cell::RefCell<AssetLoadingGroup>>>,
}

impl AssetResource {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            pending_tasks: Vec::new(),
            loading_groups: Vec::new(),
        }
    }
}
