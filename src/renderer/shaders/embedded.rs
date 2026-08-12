/// Returns embedded shader source by filename.
/// Used on wasm32 where filesystem access is unavailable.
pub fn get_embedded_shader(name: &str) -> Option<&'static str> {
    match name {
        "pbr_material.wgsl" => Some(include_str!("surface/pbr_material.wgsl")),
        "skydome.wgsl" => Some(include_str!("surface/skydome.wgsl")),
        "sprite_material.wgsl" => Some(include_str!("surface/sprite_material.wgsl")),
        "clear_cubemap.wgsl" => Some(include_str!("compute/clear_cubemap.wgsl")),
        "diffuse_irradiance.wgsl" => Some(include_str!("compute/diffuse_irradiance.wgsl")),
        "equirect_to_cubemap.wgsl" => Some(include_str!("compute/equirect_to_cubemap.wgsl")),
        "mipmap_generator.wgsl" => Some(include_str!("compute/mipmap_generator.wgsl")),
        "specular_prefilter.wgsl" => Some(include_str!("compute/specular_prefilter.wgsl")),
        _ => None,
    }
}
