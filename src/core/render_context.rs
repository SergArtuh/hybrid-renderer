pub struct RenderContext<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'a>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            device,
            queue,
            surface,
            config,
        }
    }
}
