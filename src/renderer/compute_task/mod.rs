use std::{cell::RefCell, sync::Arc};

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait},
        render_context::RenderContext,
    },
    renderer::{layout_interface::LayoutInterface, pipeline_manager::PipelineManager},
};

pub mod equirect_to_cubemap;
pub use equirect_to_cubemap::Task as EquirectToCubemapTask;
pub use equirect_to_cubemap::TaskDescriptor as EquirectToCubemapTaskDescriptor;

pub mod clear_cubemap;
pub use clear_cubemap::Task as ClearCubemapTask;
pub use clear_cubemap::TaskDescriptor as ClearCubemapTaskDescriptor;

pub mod diffuse_irradiance;
pub use diffuse_irradiance::Task as DiffuseIrradianceTask;
pub use diffuse_irradiance::TaskDescriptor as DiffuseIrradianceTaskDescriptor;

pub struct ComputeTaskFactory<'a> {
    render_context: &'a RenderContext<'a>,
    layout_interface: Arc<RefCell<LayoutInterface>>,
    pipeline_manager: &'a PipelineManager,
}

impl<'a> ComputeTaskFactory<'a> {
    pub fn new(
        render_context: &'a RenderContext<'a>,
        layout_interface: Arc<RefCell<LayoutInterface>>,
        pipeline_manager: &'a PipelineManager,
    ) -> Self {
        Self {
            render_context,
            layout_interface,
            pipeline_manager,
        }
    }

    pub fn create_task<T: ComputeTaskTrait>(&self, desc: T::Descriptor) -> ComputeTaskInstance {
        let bind_group_layout = Arc::clone(
            self.layout_interface
                .borrow()
                .compute_tasks
                .get(&T::TYPE)
                .expect("Layout not found for compute task type {T::TYPE}"),
        );
        T::create_instance(self.render_context, desc, &bind_group_layout)
            .expect("Failed to create compute task instance")
    }

    pub fn create_executor(&self) -> ComputeExecutor<'_> {
        ComputeExecutor::new(self.pipeline_manager, self.render_context)
    }
}

pub struct ComputeExecutor<'a> {
    pipeline_manager: &'a PipelineManager,
    encoder: Option<wgpu::CommandEncoder>,
}

impl<'a> ComputeExecutor<'a> {
    pub fn new(pipeline_manager: &'a PipelineManager, _render_context: &RenderContext) -> Self {
        Self {
            pipeline_manager,
            encoder: None,
        }
    }

    pub fn record(
        &mut self,
        render_context: &RenderContext,
        task_instance: &ComputeTaskInstance,
    ) -> &mut Self {
        let pipeline = self
            .pipeline_manager
            .get_compute_pipeline(task_instance.task_type);

        let encoder = self.get_or_create_encoder(render_context);
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

    pub fn execute(&mut self, render_context: &RenderContext) -> &mut Self {
        if let Some(encoder) = self.encoder.take() {
            render_context
                .queue
                .submit(std::iter::once(encoder.finish()));
        }
        self
    }

    pub fn wait(&self, render_context: &RenderContext) {
        render_context.device.poll(wgpu::Maintain::Wait);
    }

    fn get_or_create_encoder(
        &mut self,
        render_context: &RenderContext,
    ) -> &mut wgpu::CommandEncoder {
        if self.encoder.is_none() {
            self.encoder = Some(
                render_context
                    .device
                    .create_command_encoder(&Default::default()),
            );
        }
        self.encoder.as_mut().unwrap()
    }
}
