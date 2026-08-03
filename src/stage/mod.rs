pub mod frame_data;

use std::sync::Arc;

use crate::{
    assets::{camera::Camera, model::Model, skydome::Skydome},
    core::material::AlphaMode,
    core::model_node::ModelNode,
    stage::frame_data::{FrameData, RenderItem},
};

pub struct Stage {
    pub main_camera: Camera,
    models: Vec<Model>,
    skydome: Option<Skydome>,
}

impl Stage {
    pub fn new(camera: Camera) -> Self {
        Self {
            main_camera: camera,
            models: Vec::new(),
            skydome: None,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn set_skydome(&mut self, skydome: Skydome) {
        self.skydome = Some(skydome);
    }

    pub fn make_frame_data(&self) -> FrameData {
        let mut render_items = Vec::new();
        for model in &self.models {
            let mut model_render_items = Vec::new();
            Self::flatten_node(&model.root_node, &mut model_render_items, model.transform);
            render_items.extend(model_render_items);
        }

        let mut opaque_render_items = Vec::new();
        let mut blend_render_items = Vec::new();
        for render_item in &render_items {
            let alpha_mode = match &render_item {
                RenderItem::StaticMesh { material, .. } => material.alpha_mode(),
            };
            match alpha_mode {
                AlphaMode::Opaque => opaque_render_items.push(render_item.clone()),
                AlphaMode::Blend | AlphaMode::Mask => blend_render_items.push(render_item.clone()),
            }
        }
        let render_items_opaque_offset = 0;
        let render_items_transparent_offset = opaque_render_items.len();

        FrameData {
            camera_uniform: self.main_camera.get_uniform(),
            render_items_opaque: opaque_render_items,
            render_items_opaque_offset,
            render_items_transparent: blend_render_items,
            render_items_transparent_offset,
            skydome: self.skydome.clone(),
        }
    }

    fn flatten_node(
        node: &ModelNode,
        render_items: &mut Vec<RenderItem>,
        parent_world_matrix: glam::Mat4,
    ) {
        if !node.is_visible.get() {
            return;
        }

        let current_world_matrix = parent_world_matrix * node.local_transform;

        if let (Some(mesh), Some(material)) = (&node.mesh, &node.material) {
            render_items.push(RenderItem::StaticMesh {
                mesh: Arc::clone(mesh),
                material: Arc::clone(material),
                world_matrix: current_world_matrix,
            });
        }

        for child in &node.children {
            Self::flatten_node(child, render_items, current_world_matrix);
        }
    }
}
