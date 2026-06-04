use gltf::Texture;
use gltf::json::Extras;
use gltf::json::validation::Validate;
use gltf::material::Material;

#[derive(Debug, Clone)]
pub struct ClearcoatFactor(pub f32);

impl Validate for ClearcoatFactor {}

impl Default for ClearcoatFactor {
    fn default() -> Self {
        Self(0f32)
    }
}

#[derive(Debug, Clone)]
pub struct ClearcoatRoughnessFactor(pub f32);
impl Validate for ClearcoatRoughnessFactor {}

impl Default for ClearcoatRoughnessFactor {
    fn default() -> Self {
        Self(0f32)
    }
}

#[derive(Debug, Clone)]
pub struct ClearcoatTextureInfo<'a> {
    pub texture: Texture<'a>,
    pub tex_coord: u32,
}

#[derive(Debug, Clone)]
pub struct ClearcoatNormalTextureInfo<'a> {
    pub texture: Texture<'a>,
    pub tex_coord: u32,
    pub scale: f32,
}

pub struct ClearcoatDescriptor<'a> {
    pub clearcoat_factor: ClearcoatFactor,
    pub clearcoat_texture: Option<ClearcoatTextureInfo<'a>>,
    pub clearcoat_roughness_factor: ClearcoatRoughnessFactor,
    pub clearcoat_roughness_texture: Option<ClearcoatTextureInfo<'a>>,
    pub clearcoat_normal_texture: Option<ClearcoatNormalTextureInfo<'a>>,
    pub extras: Extras,
}

pub struct ExtendedMaterialDecorator<'a> {
    pub base: Material<'a>,
    pub clearcoat: Option<ClearcoatDescriptor<'a>>,
}

impl<'a> ExtendedMaterialDecorator<'a> {
    pub fn new(material: Material<'a>, document: &'a gltf::Document) -> Self {
        let clearcoat = material
            .extension_value("KHR_materials_clearcoat")
            .and_then(|value| value.as_object())
            .map(|obj| {
                let factor_f32 = obj
                    .get("clearcoatFactor")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                let roughness_f32 = obj
                    .get("clearcoatRoughnessFactor")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                let parse_tex = |key: &str| -> Option<ClearcoatTextureInfo<'a>> {
                    let tex_obj = obj.get(key)?;
                    let idx = tex_obj.get("index")?.as_u64()? as usize;
                    let tex_coord = tex_obj
                        .get("texCoord")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let gltf_texture = document.textures().nth(idx)?;
                    Some(ClearcoatTextureInfo {
                        texture: gltf_texture,
                        tex_coord,
                    })
                };

                let parse_normal_tex = |key: &str| -> Option<ClearcoatNormalTextureInfo<'a>> {
                    let tex_obj = obj.get(key)?;
                    let idx = tex_obj.get("index")?.as_u64()? as usize;
                    let tex_coord = tex_obj
                        .get("texCoord")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let scale = tex_obj.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

                    let gltf_texture = document.textures().nth(idx)?;
                    Some(ClearcoatNormalTextureInfo {
                        texture: gltf_texture,
                        tex_coord,
                        scale,
                    })
                };

                let extras = obj
                    .get("extras")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                ClearcoatDescriptor {
                    clearcoat_factor: ClearcoatFactor(factor_f32),
                    clearcoat_texture: parse_tex("clearcoatTexture"),
                    clearcoat_roughness_factor: ClearcoatRoughnessFactor(roughness_f32),
                    clearcoat_roughness_texture: parse_tex("clearcoatRoughnessTexture"),
                    clearcoat_normal_texture: parse_normal_tex("clearcoatNormalTexture"),
                    extras,
                }
            });

        Self {
            base: material,
            clearcoat,
        }
    }
}

impl<'a> std::ops::Deref for ExtendedMaterialDecorator<'a> {
    type Target = Material<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
