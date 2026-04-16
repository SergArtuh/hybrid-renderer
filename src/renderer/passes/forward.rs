use wgpu::DynamicOffset;

use crate::{
    renderer::{
        camera_manager::CameraManager, model_manager::ModelManager,
        pipeline_manager::PipelineManager,
    },
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
        pipeline_manager: &'a PipelineManager,
        camera_manager: &'a CameraManager,
        model_manager: &'a ModelManager,
        frame_data: &'a FrameData,
    ) {
        let stride = model_manager.stride as DynamicOffset;
        for (i, renderable) in frame_data.render_items.iter().enumerate() {
            match renderable {
                RenderItem::StaticMesh {
                    mesh,
                    material,
                    world_matrix: _,
                } => match (&mesh.vertex_buffer, &mesh.index_buffer) {
                    (Some(vertex_buffer), Some(index_buffer)) => {
                        let pipeline = pipeline_manager.get_pipeline(&material);

                        let offset = (i as DynamicOffset) * stride;

                        rpass.set_pipeline(&pipeline);
                        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.set_bind_group(0, &camera_manager.bind_group, &[]);
                        rpass.set_bind_group(1, &model_manager.bind_group, &[offset]);
                        rpass.set_bind_group(2, &material.bind_group(), &[]);

                        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                    (None, None) => {
                        let pipeline = pipeline_manager.get_pipeline(&material);
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &camera_manager.bind_group, &[]);
                        rpass.set_bind_group(1, &model_manager.bind_group, &[0u32]);
                        rpass.set_bind_group(2, &material.bind_group(), &[]);

                        rpass.draw(0..mesh.index_count, 0..1);
                    }
                    _ => {}
                },
            }
        }

        if let Some(skybox) = &frame_data.skybox {
            let pipeline = pipeline_manager.get_pipeline(&skybox.material);
            rpass.set_pipeline(&pipeline);
            rpass.set_bind_group(0, &camera_manager.bind_group, &[]);
            rpass.set_bind_group(1, &model_manager.bind_group, &[0u32]);
            rpass.set_bind_group(2, &skybox.material.bind_group(), &[]);
            
            match (&skybox.mesh.vertex_buffer, &skybox.mesh.index_buffer) {
                (Some(vertex_buffer), Some(index_buffer)) => {
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..skybox.mesh.index_count, 0, 0..1);
                }
                (None, None) => {
                    rpass.draw(0..skybox.mesh.index_count, 0..1);
                }
                _ => {}
            }
        }
    }
}
