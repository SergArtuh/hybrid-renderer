pub mod frame_data;

use std::sync::Arc;

use crate::{
    assets::{camera::Camera, model::Model, skybox::Skybox},
    core::{camera::CameraUniform, model_node::ModelNode},
    stage::frame_data::{FrameData, RenderItem},
};

pub struct Stage {
    pub main_camera: Camera,
    models: Vec<Model>,
    skybox: Option<Skybox>,
}

impl Stage {
    pub fn new(camera: Camera) -> Self {
        Self {
            main_camera: camera,
            models: Vec::new(),
            skybox: None,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn set_skybox(&mut self, skybox: Skybox) {
        self.skybox = Some(skybox);
    }

    pub fn make_frame_data(&self) -> FrameData {
        let mut render_items = Vec::new();
        for model in &self.models {
            let mut model_render_items = Vec::new();
            Self::flatten_node(
                &model.root_node,
                &mut model_render_items,
                glam::Mat4::IDENTITY,
            );
            render_items.extend(model_render_items);
        }
        FrameData {
            camera_uniform: CameraUniform {
                view_proj: self.main_camera.calc_matrix(),
            },
            render_items,
            skybox: self.skybox.clone(),
        }
    }

    fn flatten_node(
        node: &ModelNode,
        render_items: &mut Vec<RenderItem>,
        parent_world_matrix: glam::Mat4,
    ) {
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
