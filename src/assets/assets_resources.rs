use std::sync::mpsc::{Receiver, Sender, channel};

use crate::{
    assets::skydome::{Skydome, SkydomeDescriptor},
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

pub enum AssetData {
    Bytes(Vec<u8>),
    Texture(TextureAssetData),
    Skydome(SkydomeAssetData),
}

pub type AssetTextureCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Texture) + Send + 'static>;

pub type AssetSkydomeCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Skydome) + Send + 'static>;

pub type AssetBytesCallback =
    Box<dyn FnOnce(&mut RenderingEnvironment, &mut Stage, Vec<u8>) + Send + 'static>;

pub enum AssetCallback {
    Texture(AssetTextureCallback),
    Skydome(AssetSkydomeCallback),
    Bytes(AssetBytesCallback),
}

pub struct CompletedAssetTask {
    pub data: AssetData,
    pub callback: AssetCallback,
}

pub struct AssetResource {
    pub(crate) sender: Sender<CompletedAssetTask>,
    pub(crate) receiver: Receiver<CompletedAssetTask>,
}

impl AssetResource {
    #[warn(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}
