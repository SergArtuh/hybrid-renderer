use crate::{
    renderer::{camera_manager::CameraManager, pipeline_manager::PipelineManager},
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
        frame_data: &'a FrameData,
    ) {
        for renderable in frame_data.render_items.iter() {
            match renderable {
                RenderItem::StaticMesh {
                    mesh,
                    material,
                    world_matrix,
                } => {
                    let pipeline = pipeline_manager.get_pipeline(&material);

                    rpass.set_pipeline(&pipeline);
                    rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.set_bind_group(0, &camera_manager.bind_group, &[]);

                    rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            }
        }
    }
}
