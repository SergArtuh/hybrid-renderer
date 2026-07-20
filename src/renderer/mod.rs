use crate::{
    core::render_context::RenderContext,
    renderer::{
        compute_task::initialize_compute_tasks, materials::initialize_materials,
        pipeline_resources::PipelineResources, render_resources::RenderResources,
    },
    util::shader_watcher::ShaderWatcherResources,
};

pub mod compute_task;
pub mod frame_target;
pub mod materials;
pub mod passes;
pub mod pipeline_resources;
pub mod pipeline_system;
pub mod render_resources;
pub mod render_system;

pub use render_system::RendererSystem;

pub struct RenderingEnvironment<'a> {
    pub render_context: RenderContext<'a>,
    pub pipeline_resources: PipelineResources,
    pub render_resources: RenderResources,
    pub shader_watcher_resources: Option<ShaderWatcherResources>,
}

impl<'a> RenderingEnvironment<'a> {
    fn new(render_context: RenderContext<'a>) -> Self {
        let pipeline_resources = PipelineResources::new(&render_context);
        let render_resources = RenderResources::new(&render_context, &pipeline_resources);

        let shader_watcher_resources = if cfg!(feature = "shader-hot-reload") {
            Some(ShaderWatcherResources::create_and_initialize(
                &pipeline_resources,
            ))
        } else {
            None
        };
        Self {
            render_context,
            pipeline_resources,
            render_resources,
            shader_watcher_resources,
        }
    }
    pub fn create_and_initialize(render_context: RenderContext<'a>) -> Self {
        let mut env = Self::new(render_context);
        initialize_materials(&mut env);
        initialize_compute_tasks(&mut env);
        env
    }
}
