use crate::core::model::Model;

#[derive(Clone)]
pub struct ModelInstance<'a> {
    pub model: &'a Model, // Arc<Model>
    pub transform: glam::Mat4,
}

impl<'a> ModelInstance<'a> {
    pub fn new(model: &'a Model, transform: glam::Mat4) -> Self {
        Self { model, transform }
    }
}
