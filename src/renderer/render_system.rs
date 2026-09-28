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
            rendering_env.render_resources.prepass.execute(
                &mut frame_target,
                rendering_env,
                frame_data,
            );

            rendering_env.render_resources.forward_pass.execute(
                &mut frame_target,
                rendering_env,
                frame_data,
            );

            rendering_env.render_resources.postprocess_pass.execute(
                &mut frame_target,
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
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout => {
                eprintln!("Timeout acquired surface texture");
                panic!("Timeout acquired surface texture");
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                eprintln!("Surface is outdated");
                panic!("Surface is outdated");
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                eprintln!("Surface is lost");
                panic!("Surface is lost");
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("Surface texture validation failed");
                panic!("Surface texture validation failed");
            }
            other => {
                panic!("Failed to get current texture: {:?}", other);
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //let texture_size = surface_texture.texture.size();

        let normal_buffer_view = rendering_environment
            .render_resources
            .normal_buffer_view
            .clone();

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
            .update_buffer(
                render_context,
                &frame_data.render_items_transparent,
                frame_data.render_items_transparent_offset,
            );

        rendering_environment
            .render_resources
            .model_resources
            .update_buffer(
                render_context,
                &frame_data.render_items_opaque,
                frame_data.render_items_opaque_offset,
            );

        FrameTarget {
            surface_texture,
            view,
            normal_buffer_view,
            encoder,
        }
    }

    fn end_frame(render_context: &RenderContext, target: FrameTarget) {
        render_context
            .queue
            .submit(std::iter::once(target.encoder.finish()));
        //render_context.device.poll(wgpu::Maintain::Wait);
        render_context.queue.present(target.surface_texture);
    }
}
