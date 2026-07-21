use wgpu::DynamicOffset;

use crate::{
    renderer::{RenderingEnvironment, pipeline_system::PipelineSystem},
    stage::frame_data::{FrameData, RenderItem},
};

#[derive(Default)]
pub struct ForwardPass;

impl ForwardPass {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        render_env: &'a RenderingEnvironment,
        frame_data: &'a FrameData,
    ) {
        let stride = render_env.render_resources.model_resources.stride as DynamicOffset;
        for (i, renderable) in frame_data.render_items.iter().enumerate() {
            match renderable {
                RenderItem::StaticMesh {
                    mesh,
                    material,
                    world_matrix: _,
                } => match (&mesh.vertex_buffer, &mesh.index_buffer) {
                    (Some(vertex_buffer), Some(index_buffer)) => {
                        if let Some(pipeline) =
                            PipelineSystem::get_default_pipeline(render_env, material)
                        {
                            let offset = (i as DynamicOffset) * stride;

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
                            rpass.set_bind_group(2, &material.bind_group(), &[]);

                            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                        }
                    }
                    (None, None) => {
                        if let Some(pipeline) =
                            PipelineSystem::get_default_pipeline(render_env, material)
                        {
                            rpass.set_pipeline(&pipeline);
                            rpass.set_bind_group(
                                0,
                                &render_env.render_resources.global_resources.bind_group,
                                &[],
                            );
                            rpass.set_bind_group(
                                1,
                                &render_env.render_resources.model_resources.bind_group,
                                &[0u32],
                            );
                            rpass.set_bind_group(2, &material.bind_group(), &[]);

                            rpass.draw(0..mesh.index_count, 0..1);
                        }
                    }
                    _ => {}
                },
            }
        }

        if let Some(skydome) = &frame_data.skydome {
            if let Some(pipeline) =
                PipelineSystem::get_default_pipeline(render_env, &skydome.material)
            {
                rpass.set_pipeline(&pipeline);
                rpass.set_bind_group(
                    0,
                    &render_env.render_resources.global_resources.bind_group,
                    &[],
                );
                rpass.set_bind_group(
                    1,
                    &render_env.render_resources.model_resources.bind_group,
                    &[0u32],
                );
                rpass.set_bind_group(2, &skydome.material.bind_group(), &[]);

                match (&skydome.mesh.vertex_buffer, &skydome.mesh.index_buffer) {
                    (Some(vertex_buffer), Some(index_buffer)) => {
                        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.draw_indexed(0..skydome.mesh.index_count, 0, 0..1);
                    }
                    (None, None) => {
                        rpass.draw(0..skydome.mesh.index_count, 0..1);
                    }
                    _ => {}
                }
            }
        }
    }
}
