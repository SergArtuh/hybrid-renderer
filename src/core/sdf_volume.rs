use crate::core::uniforms::SDFUniform;

#[derive(Clone)]
pub struct SdfVolume {
    pub texture_view: wgpu::TextureView,
    pub world_to_sdf_matrix: glam::Mat4,
    pub sdf_uniform: SDFUniform,
}
