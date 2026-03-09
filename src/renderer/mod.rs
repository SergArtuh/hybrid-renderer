use crate::{
    core::{material::PhysicalMaterial, render_context::RenderContext, vertex::Vertex},
    renderer::{
        camera_manager::CameraManager, frame_target::FrameTarget,
        layout_interface::LayoutInterface, model_manager::ModelManager,
        pipeline_manager::PipelineManager,
    },
    stage::frame_data::FrameData,
};

pub mod camera_manager;
pub mod frame_target;
pub mod layout_interface;
pub mod model_manager;
pub mod passes;
pub mod pipeline_manager;

use passes::forward::ForwardPass;

pub struct Renderer {
    forward_pass: ForwardPass,
    camera_manager: CameraManager,
    model_manager: ModelManager,
    pipeline_manager: PipelineManager,
    layout_interface: LayoutInterface,
    depth_texture_view: wgpu::TextureView,
}

impl Renderer {
    pub fn new(render_context: &RenderContext) -> Self {
        let layout_interface = LayoutInterface::new(render_context);
        let camera_manager = CameraManager::new(render_context, &layout_interface.global);
        let model_manager = ModelManager::new(render_context, &layout_interface.model);
        let depth_texture_view = Self::create_depth_texture(render_context);

        let mut pipeline_manager = PipelineManager::new();
        pipeline_manager.register_pipeline::<PhysicalMaterial>(
            render_context,
            &layout_interface,
            "pbr_material.wgsl",
        );

        let forward_pass = ForwardPass::default();

        Self {
            forward_pass,
            camera_manager,
            model_manager,
            pipeline_manager,
            layout_interface,
            depth_texture_view,
        }
    }

    pub fn render(&mut self, render_context: &RenderContext, frame_data: &FrameData) {
        self.pipeline_manager
            .check_shader_updates(render_context, &self.layout_interface);
        let mut frame_target = self.begin_frame(render_context);

        self.camera_manager
            .update_buffer(render_context, &frame_data.camera_uniform);

        self.model_manager
            .update_buffer(render_context, &frame_data.render_items);

        {
            let mut render_pass =
                self.create_render_pass(&mut frame_target.encoder, &frame_target.view);

            self.forward_pass.execute(
                &mut render_pass,
                &self.pipeline_manager,
                &self.camera_manager,
                &self.model_manager,
                frame_data,
            );
        }

        self.end_frame(render_context, frame_target);
    }

    fn begin_frame(&self, render_context: &RenderContext) -> FrameTarget {
        let surface_texture = match render_context.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to get current texture: {:?}", e);
                panic!("Failed to get current texture");
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        FrameTarget {
            surface_texture,
            view,
            encoder,
        }
    }

    fn end_frame(&self, render_context: &RenderContext, target: FrameTarget) {
        render_context
            .queue
            .submit(std::iter::once(target.encoder.finish()));
        target.surface_texture.present();
    }

    fn create_depth_texture(render_context: &RenderContext) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: render_context.config.width,
            height: render_context.config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PipelineManager::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = render_context.device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_render_pass<'a>(
        self: &'a Self,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass
    }
}
