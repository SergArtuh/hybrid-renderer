use super::texture::Texture;
use std::sync::Arc;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPrecision {
    U8,
    F32,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureChannels {
    R,
    RG,
    RGB,
    RGBA,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureFormatDescriptor {
    pub channels: TextureChannels,
    pub precision: ComponentPrecision,
    pub is_srgb: bool,
}

enum RawTextureData {
    U8(Vec<u8>),
    F32(Vec<f32>),
}

trait IntoWgpuFormat {
    fn into_format_descriptor(self) -> TextureFormatDescriptor;
}

impl IntoWgpuFormat for wgpu::TextureFormat {
    fn into_format_descriptor(self) -> TextureFormatDescriptor {
        let channels = match self {
            wgpu::TextureFormat::R8Unorm => TextureChannels::R,
            wgpu::TextureFormat::Rg8Unorm => TextureChannels::RG,
            wgpu::TextureFormat::Rgba8Unorm => TextureChannels::RGBA,
            wgpu::TextureFormat::Rgba8UnormSrgb => TextureChannels::RGBA,
            wgpu::TextureFormat::Rgba32Float => TextureChannels::RGBA,
            _ => panic!("Unsupported format to linear: {:?}", self),
        };
        let precision = match self {
            wgpu::TextureFormat::R8Unorm => ComponentPrecision::U8,
            wgpu::TextureFormat::Rg8Unorm => ComponentPrecision::U8,
            wgpu::TextureFormat::Rgba8Unorm => ComponentPrecision::U8,
            wgpu::TextureFormat::Rgba8UnormSrgb => ComponentPrecision::U8,
            wgpu::TextureFormat::Rgba32Float => ComponentPrecision::F32,
            _ => panic!("Unsupported format to linear: {:?}", self),
        };
        let is_srgb = match self {
            wgpu::TextureFormat::Rgba8UnormSrgb => true,
            _ => false,
        };
        TextureFormatDescriptor {
            channels,
            precision,
            is_srgb,
        }
    }
}

pub struct TextureBuilder<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    label: Option<&'a str>,
    format: TextureFormatDescriptor,
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
            format: TextureFormatDescriptor {
                channels: TextureChannels::RGBA,
                precision: ComponentPrecision::U8,
                is_srgb: false,
            },
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

    pub fn with_precision(mut self, precision: ComponentPrecision) -> Self {
        self.format.precision = precision;
        self
    }

    pub fn with_channels(mut self, channels: TextureChannels) -> Self {
        self.format.channels = channels;
        self
    }

    pub fn with_srgb(mut self, is_srgb: bool) -> Self {
        self.format.is_srgb = is_srgb;
        self
    }

    pub fn with_wgpu_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format.into_format_descriptor();
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

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    pub fn with_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn build(mut self) -> Texture {
        let source = std::mem::replace(&mut self.source, TextureSource::Empty);
        let (width, height, rgba) = self.resolve_image_data(source);

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let (format, channel_count, byte_per_chanel) = self.get_texture_format_description(&rgba);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: self.usage,
            view_formats: &[],
        });

        let channels = channel_count * byte_per_chanel;

        let rgba_data = match &rgba {
            RawTextureData::U8(data) => bytemuck::cast_slice(&data),
            RawTextureData::F32(data) => bytemuck::cast_slice(&data),
        };

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(channels * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

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

    fn get_texture_format_description(
        &self,
        rgba: &RawTextureData,
    ) -> (wgpu::TextureFormat, u32, u32) {
        use wgpu::TextureFormat as TF;
        let channels = match self.format.channels {
            TextureChannels::R => 1,
            TextureChannels::RG => 2,
            TextureChannels::RGB => 4,
            TextureChannels::RGBA => 4,
        };

        match rgba {
            RawTextureData::U8(_) => {
                let format = match channels {
                    1 => TF::R8Unorm,
                    2 => TF::Rg8Unorm,
                    _ => {
                        if self.format.is_srgb {
                            TF::Rgba8UnormSrgb
                        } else {
                            TF::Rgba8Unorm
                        }
                    }
                };
                (format, channels, 1)
            }
            RawTextureData::F32(_) => {
                let format = match channels {
                    1 => TF::R32Float,
                    2 => TF::Rg32Float,
                    _ => TF::Rgba32Float,
                };
                (format, channels, 4)
            }
        }
    }

    fn resolve_image_data(&self, source: TextureSource) -> (u32, u32, RawTextureData) {
        match source {
            TextureSource::Bytes(bytes) => {
                let img = image::load_from_memory(bytes).expect("Failed to load image from memory");
                self.resolve_dynamic_image_data(img)
            }
            TextureSource::Image(img) => self.resolve_dynamic_image_data(img),
            TextureSource::Raw {
                pixels,
                width,
                height,
            } => self.resolve_raw_texture_data(pixels, width, height),
            TextureSource::Empty => self.resolve_empty_texture_data(),
        }
    }

    fn resolve_dynamic_image_data(&self, img: image::DynamicImage) -> (u32, u32, RawTextureData) {
        let color_type = if self.format.precision == ComponentPrecision::Auto {
            Some(dbg!(img.color()))
        } else {
            match self.format.precision {
                ComponentPrecision::F32 => Some(image::ColorType::Rgba32F),
                ComponentPrecision::U8 => Some(image::ColorType::Rgba8),
                _ => panic!("Unsupported precision: {:?}", self.format.precision),
            }
        };

        match color_type.unwrap() {
            image::ColorType::Rgba32F | image::ColorType::Rgb32F => {
                let rgba = img.to_rgba32f();
                let (w, h) = rgba.dimensions();
                (w, h, RawTextureData::F32(rgba.into_raw()))
            }
            image::ColorType::Rgba8 | image::ColorType::Rgb8 => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                (w, h, RawTextureData::U8(rgba.into_raw()))
            }
            _ => panic!("Unsupported color type: {:?}", color_type),
        }
    }

    fn resolve_raw_texture_data(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> (u32, u32, RawTextureData) {
        if self.format.precision == ComponentPrecision::Auto
            || self.format.precision == ComponentPrecision::U8
        {
            match self.format.channels {
                TextureChannels::RGB => {
                    let mut rgba = Vec::with_capacity(pixels.len() / 3 * 4);
                    for rgb in pixels.chunks_exact(3) {
                        rgba.extend_from_slice(rgb);
                        rgba.push(255);
                    }
                    (width, height, RawTextureData::U8(rgba))
                }
                TextureChannels::R => (width, height, RawTextureData::U8(pixels.to_vec())),
                TextureChannels::RG => (width, height, RawTextureData::U8(pixels.to_vec())),
                TextureChannels::RGBA => (width, height, RawTextureData::U8(pixels.to_vec())),
            }
        } else {
            panic!(
                "Unsupported precision for raw data: {:?}",
                self.format.precision
            );
        }
    }

    fn resolve_empty_texture_data(&self) -> (u32, u32, RawTextureData) {
        let (width, height) = self
            .size
            .expect("Size must be provided for empty texture if no bytes/image are provided");

        let channels = match self.format.channels {
            TextureChannels::RGB => 4,
            TextureChannels::R => 1,
            TextureChannels::RG => 2,
            TextureChannels::RGBA => 4,
        };

        if self.format.precision == ComponentPrecision::Auto
            || self.format.precision == ComponentPrecision::U8
        {
            (
                width,
                height,
                RawTextureData::U8(vec![0; (width * height * channels) as usize]),
            )
        } else {
            (
                width,
                height,
                RawTextureData::F32(vec![0.0f32; (width * height * channels) as usize]),
            )
        }
    }
}
