pub mod frame_data;

use crate::{
    assets::{camera::Camera, model_instance::ModelInstance},
    core::camera::CameraUniform,
    stage::frame_data::FrameData,
};

pub struct Stage<'a> {
    pub main_camera: Camera,
    pub model_instances: Vec<ModelInstance<'a>>,
}

impl<'a> Stage<'a> {
    pub fn new(camera: Camera) -> Self {
        Self {
            main_camera: camera,
            model_instances: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model: ModelInstance<'a>) {
        self.model_instances.push(model);
    }

    pub fn make_frame_data(&self) -> FrameData<'a> {
        FrameData {
            camera_uniform: CameraUniform {
                view_proj: self.main_camera.calc_matrix(),
            },
            model_instances: self.model_instances.clone(),
        }
    }
}
