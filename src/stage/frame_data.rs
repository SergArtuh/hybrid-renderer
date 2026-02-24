use crate::{assets::model_instance::ModelInstance, core::camera::CameraUniform};

pub struct FrameData<'a> {
    pub camera_uniform: CameraUniform,
    pub model_instances: Vec<ModelInstance<'a>>,
}
