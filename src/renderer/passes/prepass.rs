use wgpu::DynamicOffset;

use crate::{
    core::material::{AlphaMode, MaterialType, PipelineKey},
    renderer::{RenderingEnvironment, frame_target::FrameTarget, pipeline_system::PipelineSystem},
    stage::frame_data::{FrameData, RenderItem},
};

#[derive(Default)]
pub struct PrePass;

impl PrePass {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute<'a>(
        &'a self,
        frame_target: &'a mut FrameTarget,
        // rpass: &mut wgpu::RenderPass<'a>,
        render_env: &'a RenderingEnvironment,
        frame_data: &'a FrameData,
    ) {
        let mut rpass = Self::create_render_pass(
            render_env,
            &mut frame_target.encoder,
            &frame_target.normal_view,
        );

        let stride = render_env.render_resources.model_resources.stride as DynamicOffset;
        for alpha_mode in [AlphaMode::Opaque].iter() {
            let (render_items, base_offset) = if *alpha_mode == AlphaMode::Blend {
                (
                    &frame_data.render_items_transparent,
                    frame_data.render_items_transparent_offset,
                )
            } else {
                (
                    &frame_data.render_items_opaque,
                    frame_data.render_items_opaque_offset,
                )
            };
            for (i, renderable) in render_items.iter().enumerate() {
                match renderable {
                    RenderItem::StaticMesh {
                        mesh,
                        material: _,
                        world_matrix: _,
                    } => match (&mesh.vertex_buffer, &mesh.index_buffer) {
                        (Some(vertex_buffer), Some(index_buffer)) => {
                            if let Some(pipeline) = PipelineSystem::get_pipeline(
                                render_env,
                                MaterialType::PrePass,
                                PipelineKey::default(),
                            ) {
                                let offset = ((base_offset + i) as DynamicOffset) * stride;

                                rpass.set_pipeline(&pipeline);
                                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                                rpass.set_index_buffer(
                                    index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                rpass.set_bind_group(
                                    0,
                                    &render_env.render_resources.global_resources.bind_group,
                                    &[],
                                );
                                rpass.set_bind_group(
                                    1,
                                    &render_env.render_resources.model_resources.bind_group,
                                    &[offset],
                                );

                                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    fn create_render_pass<'a>(
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
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
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
            multiview_mask: None,
        });
        render_pass
    }
}
