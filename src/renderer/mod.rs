#[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
use crate::util::shader_watcher::ShaderWatcherResources;
use crate::{
    assets::assets_resources::AssetResource,
    core::render_context::RenderContext,
    renderer::{
        compute_task::initialize_compute_tasks, materials::initialize_materials,
        pipeline_resources::PipelineResources, render_resources::RenderResources,
    },
};

pub mod compute_task;
pub mod frame_target;
pub mod materials;
pub mod passes;
pub mod pipeline_resources;
pub mod pipeline_system;
pub mod render_resources;
pub mod render_system;
pub mod shaders;

pub use render_system::RendererSystem;

pub struct RenderingEnvironment<'a> {
    pub render_context: RenderContext<'a>,
    pub pipeline_resources: PipelineResources,
    pub render_resources: RenderResources,
    pub asset_resources: AssetResource,
    #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
    pub shader_watcher_resources: Option<ShaderWatcherResources>,
}

impl<'a> RenderingEnvironment<'a> {
    fn new(render_context: RenderContext<'a>) -> Self {
        let pipeline_resources = PipelineResources::new(&render_context);
        let render_resources = RenderResources::new(&render_context, &pipeline_resources);
        let asset_resources = AssetResource::new();

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        let shader_watcher_resources = Some(ShaderWatcherResources::create_and_initialize(
            &pipeline_resources,
        ));

        Self {
            render_context,
            pipeline_resources,
            render_resources,
            asset_resources,
            #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
            shader_watcher_resources,
        }
    }
    pub fn create_and_initialize(render_context: RenderContext<'a>) -> Self {
        let mut env = Self::new(render_context);
        initialize_materials(&mut env);
        initialize_compute_tasks(&mut env);
        env
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.render_context.resize(width, height);
        self.render_resources.resize(&self.render_context);
    }
}
