use std::sync::Arc;

use crate::{
    core::render_context::RenderContext,
    renderer::{RenderingEnvironment, frame_target::FrameTarget},
    stage::frame_data::FrameData,
};

pub struct RendererSystem {}

impl<'a> RendererSystem {
    pub fn render(rendering_env: &mut RenderingEnvironment, frame_data: &FrameData) {
        let mut frame_target = Self::begin_frame(rendering_env, frame_data);

        {
            let mut render_pass = Self::create_render_pass(
                rendering_env,
                &mut frame_target.encoder,
                &frame_target.view,
            );

            rendering_env.render_resources.forward_pass.execute(
                &mut render_pass,
                rendering_env,
                frame_data,
            );
        }

        Self::end_frame(&rendering_env.render_context, frame_target);
    }

    fn begin_frame(
        rendering_environment: &mut RenderingEnvironment,
        frame_data: &FrameData,
    ) -> FrameTarget {
        let render_context = &rendering_environment.render_context;
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

        //pipeline_system.check_shader_updates(render_context, pipeline_resources);

        rendering_environment
            .render_resources
            .global_resources
            .update_camera_uniform_buffer(render_context, &frame_data.camera_uniform);

        frame_data
            .get_environment_texture()
            .map(|environment_texture| {
                rendering_environment
                    .render_resources
                    .global_resources
                    .update_skybox_texture(
                        render_context,
                        &rendering_environment.pipeline_resources.global,
                        Arc::clone(&environment_texture.skybox),
                        Arc::clone(&environment_texture.irradiance),
                        Arc::clone(&environment_texture.specular),
                        &rendering_environment.render_resources.common_sampler,
                    )
            });

        rendering_environment
            .render_resources
            .model_resources
            .update_buffer(render_context, &frame_data.render_items);

        FrameTarget {
            surface_texture,
            view,
            encoder,
        }
    }

    fn end_frame(render_context: &RenderContext, target: FrameTarget) {
        render_context
            .queue
            .submit(std::iter::once(target.encoder.finish()));
        //render_context.device.poll(wgpu::Maintain::Wait);
        target.surface_texture.present();
    }

    fn create_render_pass(
        rendering_environment: &'a RenderingEnvironment,
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
                view: &rendering_environment.render_resources.depth_texture_view,
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
