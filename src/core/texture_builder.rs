use std::sync::Arc;

use gltf::image::Format;

use super::texture::Texture;

pub trait IntoWgpuFormat {
    fn into_wgpu_srgb(self) -> wgpu::TextureFormat;
    fn into_wgpu_linear(self) -> wgpu::TextureFormat;
}

impl IntoWgpuFormat for wgpu::TextureFormat {
    fn into_wgpu_srgb(self) -> wgpu::TextureFormat {
        self
    }

    fn into_wgpu_linear(self) -> wgpu::TextureFormat {
        self
    }
}

impl IntoWgpuFormat for Format {
    fn into_wgpu_srgb(self) -> wgpu::TextureFormat {
        match self {
            Format::R8G8B8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            _ => panic!("Unsupported format to srgb: {:?}", self),
        }
    }

    fn into_wgpu_linear(self) -> wgpu::TextureFormat {
        match self {
            Format::R8 => wgpu::TextureFormat::R8Unorm,
            Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
            Format::R8G8B8 => wgpu::TextureFormat::Rgba8Unorm,
            Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8Unorm,
            Format::R16 => wgpu::TextureFormat::R16Unorm,
            Format::R16G16 => wgpu::TextureFormat::Rg16Unorm,
            Format::R16G16B16 => wgpu::TextureFormat::Rgba16Unorm,
            Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Unorm,
            _ => panic!("Unsupported format to linear: {:?}", self),
        }
    }
}

pub enum TextureSource<'a> {
    Empty,
    Bytes(&'a [u8]),
    Image(image::DynamicImage),
    Raw {
        pixels: &'a [u8],
        width: u32,
        height: u32,
    },
}

pub struct TextureBuilder<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    label: Option<&'a str>,
    format: wgpu::TextureFormat,
    mag_filter: wgpu::FilterMode,
    min_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
    address_mode: wgpu::AddressMode,
    source: TextureSource<'a>,
    size: Option<(u32, u32)>,
    usage: wgpu::TextureUsages,
}

impl<'a> TextureBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, queue: &'a wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            label: None,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode: wgpu::AddressMode::ClampToEdge,
            source: TextureSource::Empty,
            size: None,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }
    }

    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_format<F: IntoWgpuFormat>(mut self, format: F, srgb: bool) -> Self {
        self.format = if srgb {
            format.into_wgpu_srgb()
        } else {
            format.into_wgpu_linear()
        };
        self
    }

    pub fn with_filter(mut self, mag: wgpu::FilterMode, min: wgpu::FilterMode) -> Self {
        self.mag_filter = mag;
        self.min_filter = min;
        self
    }

    pub fn with_mipmap_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.mipmap_filter = filter;
        self
    }

    pub fn with_address_mode(mut self, mode: wgpu::AddressMode) -> Self {
        self.address_mode = mode;
        self
    }

    pub fn from_bytes(mut self, bytes: &'a [u8]) -> Self {
        self.source = TextureSource::Bytes(bytes);
        self
    }

    pub fn from_image(mut self, image: image::DynamicImage) -> Self {
        self.source = TextureSource::Image(image);
        self
    }

    pub fn from_raw(mut self, pixels: &'a [u8], width: u32, height: u32) -> Self {
        self.source = TextureSource::Raw {
            pixels,
            width,
            height,
        };
        self
    }

    // TODO: Remove this method
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    pub fn with_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn build(self) -> Texture {
        let (width, height, rgba) = match self.source {
            TextureSource::Bytes(bytes) => {
                let img = image::load_from_memory(bytes).expect("Failed to load image from memory");
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                (w, h, Some(rgba.into_raw()))
            }
            TextureSource::Image(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                (w, h, Some(rgba.into_raw()))
            }
            TextureSource::Raw {
                pixels,
                width,
                height,
            } => {
                let src_channels = pixels.len() / (width * height) as usize;
                match src_channels {
                    3 => {
                        let rgba = pixels
                            .chunks_exact(3)
                            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                            .collect();
                        (width, height, Some(rgba))
                    }
                    1 => (width, height, Some(pixels.to_vec())),
                    2 => (width, height, Some(pixels.to_vec())),
                    4 => (width, height, Some(pixels.to_vec())),
                    _ => panic!("Unsupported raw texture channel count: {}", src_channels),
                }
            }

            // TODO: Remove this case
            TextureSource::Empty => {
                let (w, h) = self.size.expect(
                    "Size must be provided for empty texture if no bytes/image are provided",
                );
                (w, h, None)
            }
        };

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        println!("\n\n\n");
        println!("Texture size: {} {}", width, height);
        println!("Texture format: {:?}", self.format);
        println!("Texture usage: {:?}", self.usage);
        println!("Texture label: {:?}", self.label);
        println!("Texture rgba: {}", rgba.is_some());

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        });

        let block_size = Self::get_block_size(self.format);
        if let Some(rgba) = rgba {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(block_size * width),
                    rows_per_image: Some(height),
                },
                texture_size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: self.address_mode,
            address_mode_v: self.address_mode,
            address_mode_w: self.address_mode,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: self.mipmap_filter,
            ..Default::default()
        });

        Texture {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: Arc::new(sampler),
            width,
            height,
        }
    }

    fn get_block_size(format: wgpu::TextureFormat) -> u32 {
        match format {
            wgpu::TextureFormat::R8Unorm => 1,
            wgpu::TextureFormat::Rg8Unorm => 2,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Rgba16Unorm => 8,
            _ => 4,
        }
    }
}
