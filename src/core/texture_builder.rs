use super::texture::Texture;

pub enum TextureSource<'a> {
    Empty,
    Bytes(&'a [u8]),
    Image(image::DynamicImage),
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

    pub fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
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
                (w, h, Some(rgba))
            }
            TextureSource::Image(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                (w, h, Some(rgba))
            }
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
                    bytes_per_row: Some(4 * width),
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
            texture,
            view,
            sampler,
            width,
            height,
        }
    }
}
