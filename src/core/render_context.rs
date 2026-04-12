use crate::core::texture::DefaultTextures;

pub struct RenderContext<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
    pub default_textures: DefaultTextures,
    pub common_sampler: wgpu::Sampler,
    //pub common_linear_sampler: wgpu::Sampler,
    pub common_nearest_sampler: wgpu::Sampler,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'a>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let default_textures = DefaultTextures::new(&device, &queue);
        let common_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let common_nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            device,
            queue,
            surface,
            config,
            default_textures,
            common_sampler,
            common_nearest_sampler,
        }
    }
}
