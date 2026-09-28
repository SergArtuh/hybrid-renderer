use crate::{
    core::material::MaterialType,
    renderer::{RenderingEnvironment, frame_target::FrameTarget, pipeline_system::PipelineSystem},
    stage::frame_data::FrameData,
};

#[derive(Default)]
pub struct PostProcessPass;

impl PostProcessPass {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute<'a>(
        &'a self,
        frame_target: &'a mut FrameTarget,
        render_env: &'a RenderingEnvironment,
        _frame_data: &'a FrameData,
    ) {
        let mut rpass =
            Self::create_render_pass(render_env, &mut frame_target.encoder, &frame_target.view);

        if let Some(pipeline) =
            PipelineSystem::get_default_pipeline(render_env, MaterialType::PostProcess)
        {
            let resources = &render_env.render_resources.postprocess_resources;
            let material_borrow = resources.material.borrow();
            if let Some(material) = material_borrow.as_ref() {
                let mesh = &resources.mesh;

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
                rpass.set_bind_group(2, material.bind_group(), &[]);

                let count = if mesh.index_count > 0 {
                    mesh.index_count
                } else {
                    6
                };
                rpass.draw(0..count, 0..1);
            }
        }
    }

    fn create_render_pass<'a>(
        _rendering_environment: &'a RenderingEnvironment,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        render_pass
    }
}
