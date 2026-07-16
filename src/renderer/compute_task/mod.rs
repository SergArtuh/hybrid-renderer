use std::{path::PathBuf, sync::Arc};

use crate::{
    core::compute_task::{ComputeTaskInstance, ComputeTaskTrait},
    renderer::{RenderingEnvironment, pipeline_system::PipelineSystem},
};

pub mod equirect_to_cubemap;
pub use equirect_to_cubemap::Provider as EquirectToCubemapProvider;
pub use equirect_to_cubemap::Task as EquirectToCubemapTask;
pub use equirect_to_cubemap::TaskDescriptor as EquirectToCubemapTaskDescriptor;

pub mod clear_cubemap;
pub use clear_cubemap::Task as ClearCubemapTask;
pub use clear_cubemap::TaskDescriptor as ClearCubemapTaskDescriptor;

pub mod diffuse_irradiance;
pub use diffuse_irradiance::Provider as DiffuseIrradianceProvider;
pub use diffuse_irradiance::Task as DiffuseIrradianceTask;
pub use diffuse_irradiance::TaskDescriptor as DiffuseIrradianceTaskDescriptor;

pub mod specular_prefilter;
pub use specular_prefilter::Provider as SpecularPrefilterProvider;
pub use specular_prefilter::Task as SpecularPrefilterTask;
pub use specular_prefilter::TaskDescriptor as SpecularPrefilterTaskDescriptor;

pub mod mipmap_generator;
pub use mipmap_generator::Provider as MipmapGeneratorProvider;
pub use mipmap_generator::Task as MipmapGeneratorTask;
pub use mipmap_generator::TaskDescriptor as MipmapGeneratorTaskDescriptor;

pub fn initialize_compute_tasks(render_env: &mut RenderingEnvironment) {
    render_env.pipeline_resources.base_compute_shaders_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders")
            .join("compute");

    macro_rules! register {
        ($($t:ty),* $(,)?) => {
            $( crate::renderer::pipeline_system::PipelineSystem::register_compute_pipeline::<$t>(render_env); )*
        };
    }

    register![
        EquirectToCubemapTask,
        ClearCubemapTask,
        DiffuseIrradianceTask,
        MipmapGeneratorTask,
        SpecularPrefilterTask
    ];
}

pub struct ComputeTaskFactory<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

impl<'a> ComputeTaskFactory<'a> {
    pub fn new(render_env: &'a RenderingEnvironment) -> Self {
        Self { render_env }
    }

    pub fn create_task<T: ComputeTaskTrait>(&self, desc: T::Descriptor) -> ComputeTaskInstance {
        let bind_group_layout = Arc::clone(
            self.render_env
                .pipeline_resources
                .compute_tasks
                .get(&T::TYPE)
                .expect("Layout not found for compute task type {T::TYPE}"),
        );
        T::create_instance(&self.render_env, desc, &bind_group_layout)
            .expect("Failed to create compute task instance")
    }

    pub fn create_executor(&self) -> ComputeExecutor<'_> {
        ComputeExecutor::new(&self.render_env)
    }

    pub fn create_executor_from_encoder(
        &self,
        encoder: wgpu::CommandEncoder,
    ) -> ComputeExecutor<'_> {
        ComputeExecutor::from_encoder(&self.render_env, encoder)
    }
}

pub struct ComputeExecutor<'a> {
    render_env: &'a RenderingEnvironment<'a>,
    encoder: Option<wgpu::CommandEncoder>,
}

impl<'a> ComputeExecutor<'a> {
    pub fn new(render_env: &'a RenderingEnvironment<'a>) -> Self {
        Self {
            render_env,
            encoder: None,
        }
    }

    pub fn from_encoder(
        render_env: &'a RenderingEnvironment<'a>,
        encoder: wgpu::CommandEncoder,
    ) -> Self {
        Self {
            render_env,
            encoder: Some(encoder),
        }
    }

    pub fn take_encoder(&mut self) -> Option<wgpu::CommandEncoder> {
        self.encoder.take()
    }

    pub fn record(&mut self, task_instance: &ComputeTaskInstance) -> &mut Self {
        let pipeline =
            PipelineSystem::get_compute_pipeline(self.render_env, task_instance.task_type);

        let encoder = self.get_or_create_encoder();
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &task_instance.bind_group, &[]);
            cpass.dispatch_workgroups(
                task_instance.dispatch_size.0,
                task_instance.dispatch_size.1,
                task_instance.dispatch_size.2,
            );
        }

        self
    }

    pub fn execute(&mut self) -> &mut Self {
        if let Some(encoder) = self.encoder.take() {
            self.render_env
                .render_context
                .queue
                .submit(std::iter::once(encoder.finish()));
        }
        self
    }

    pub fn wait(&self) {
        self.render_env
            .render_context
            .device
            .poll(wgpu::Maintain::Wait);
    }

    pub fn get_encoder_mut(&mut self) -> Option<&mut wgpu::CommandEncoder> {
        self.encoder.as_mut()
    }

    fn get_or_create_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        if self.encoder.is_none() {
            self.encoder = Some(
                self.render_env
                    .render_context
                    .device
                    .create_command_encoder(&Default::default()),
            );
        }
        self.encoder.as_mut().unwrap()
    }
}
