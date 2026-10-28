use crate::{
    renderer::{
        RenderingEnvironment, compute_task::SDFIlluminationProvider, frame_target::FrameTarget,
    },
    stage::frame_data::FrameData,
};

pub struct SDFPass;

impl SDFPass {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute<'a>(
        &'a self,
        _frame_target: &'a mut FrameTarget,
        render_env: &'a RenderingEnvironment,
        _frame_data: &'a FrameData,
    ) {
        for sdf_volume in &_frame_data.sdf_volumes {
            let _ = SDFIlluminationProvider::new(render_env).process(
                &render_env.render_resources.normal_buffer_view,
                &render_env.render_resources.depth_texture_view,
                &render_env.render_resources.sdf_ao_buffer_view,
                &sdf_volume.texture_view,
                &sdf_volume.sdf_uniform,
                render_env.render_context.config.width,
                render_env.render_context.config.height,
            );
        }
    }
}
