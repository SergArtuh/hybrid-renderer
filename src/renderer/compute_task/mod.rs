use std::{cell::RefCell, sync::Arc};

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait},
        render_context::RenderContext,
    },
    renderer::{layout_interface::LayoutInterface, pipeline_manager::PipelineManager},
};

pub mod equirect_to_cubemap;

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
        ComputeExecutor {
            pipeline_manager: &self.pipeline_manager,
        }
    }
}

pub struct ComputeExecutor<'a> {
    pipeline_manager: &'a PipelineManager,
}

impl<'a> ComputeExecutor<'a> {
    pub fn new(pipeline_manager: &'a PipelineManager) -> Self {
        Self { pipeline_manager }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, task_instance: &ComputeTaskInstance) {
        let pipeline = self
            .pipeline_manager
            .get_compute_pipeline(task_instance.task_type);
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(pipeline);
        cpass.set_bind_group(0, &task_instance.bind_group, &[]);
        cpass.dispatch_workgroups(
            task_instance.dispatch_size.0,
            task_instance.dispatch_size.1,
            task_instance.dispatch_size.2,
        );
    }

    pub fn execute_immediate(
        &self,
        render_context: &RenderContext,
        instance: &ComputeTaskInstance,
    ) {
        let mut encoder = render_context
            .device
            .create_command_encoder(&Default::default());
        self.record(&mut encoder, instance);
        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        render_context.device.poll(wgpu::Maintain::Wait);
    }
}
