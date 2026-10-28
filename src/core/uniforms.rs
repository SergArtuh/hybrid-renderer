use serde::Deserialize;

use crate::core::math::{Mat4, Vec4};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: Mat4,
    pub inv_skybox_view_proj: Mat4,
    pub position: Vec4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model_matrix: Mat4,
    pub normal_matrix: Mat4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkydomeUniform {
    // [radius, dome_factor, 0.0, 0.0]
    pub data: [f32; 4],
}

impl SkydomeUniform {
    pub fn new(radius: f32, dome_factor: f32) -> Self {
        Self {
            data: [radius, dome_factor, 0.0, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrMaterialUniforms {
    pub base_color_factor: [f32; 4],
    pub emissive_and_scale: [f32; 4],
    pub pbr_factors: [f32; 4],
    pub clearcoat_factors: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpecularPrefilterUniform {
    // [roughness, 0.0, 0.0, 0.0]
    pub data: [f32; 4],
}

impl SpecularPrefilterUniform {
    pub fn new(roughness: f32) -> Self {
        Self {
            data: [roughness, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct RawSdfConfig {
    pub grid_resolution: u32,
    pub sdf_inv_world_matrix: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SDFUniform {
    pub inv_world: Mat4,
    pub tiles_per_side: u32,
    pub _padding: [u32; 3],
}

impl SDFUniform {
    pub fn new(inv_world: Mat4, tiles_per_side: u32) -> Self {
        Self {
            inv_world,
            tiles_per_side,
            _padding: [0; 3],
        }
    }
}
