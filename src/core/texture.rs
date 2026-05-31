use std::sync::Arc;

use crate::core::texture_builder::TextureBuilder;

pub struct DefaultTextures {
    pub white: Arc<wgpu::TextureView>,
    pub black: Arc<wgpu::TextureView>,
    pub normal: Arc<wgpu::TextureView>,
    pub black_metallic: Arc<wgpu::TextureView>,
    pub cubemap: Arc<Texture>,
}

impl DefaultTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let create_color_img = |r, g, b, a| {
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                1,
                1,
                image::Rgba([r, g, b, a]),
            ))
        };

        let white = TextureBuilder::new(device, queue)
            .with_label("white_fallback")
            .from_image(create_color_img(255, 255, 255, 255))
            .build();

        let black = TextureBuilder::new(device, queue)
            .with_label("black_fallback")
            .from_image(create_color_img(0, 0, 0, 255))
            .build();

        let normal = TextureBuilder::new(device, queue)
            .with_label("normal_fallback")
            .from_image(create_color_img(128, 128, 255, 255))
            .build();

        let black_metallic = TextureBuilder::new(device, queue)
            .with_label("black_metallic_fallback")
            .from_image(create_color_img(255, 255, 255, 255))
            .build();

        let cubemap = TextureBuilder::new(device, queue)
            .with_label("cubemap_fallback")
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_size(1, 1)
            .as_cubemap()
            .build();

        Self {
            white: Arc::clone(&white.view),
            black: Arc::clone(&black.view),
            normal: Arc::clone(&normal.view),
            black_metallic: Arc::clone(&black_metallic.view),
            cubemap: Arc::new(cubemap),
        }
    }
}

pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub array_view: Option<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
    pub width: u32,
    pub height: u32,
}
