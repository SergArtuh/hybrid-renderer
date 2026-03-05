use crate::{
    renderer::camera_manager::CameraManager, renderer::pipeline_manager::PipelineManager,
    stage::frame_data::FrameData,
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
        for model_instance in frame_data.model_instances.iter() {
            let pipeline = pipeline_manager.get_pipeline(&model_instance.model.material);

            rpass.set_pipeline(&pipeline);
            rpass.set_vertex_buffer(0, model_instance.model.mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(
                model_instance.model.mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.set_bind_group(0, &camera_manager.bind_group, &[]);

            rpass.draw_indexed(0..model_instance.model.mesh.index_count, 0, 0..1);
        }
    }
}
