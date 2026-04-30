use crate::core::render_context::RenderContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeTaskType {
    EquirectToCubemap,
}

pub struct ComputeTaskInstance {
    pub task_type: ComputeTaskType,
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

pub trait ComputeTaskTrait: Sized {
    type Descriptor;
    const TYPE: ComputeTaskType;

    fn create_instance(
        _render_context: &RenderContext,
        _desc: Self::Descriptor,
        _layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        todo!()
    }

    fn get_shader_path() -> &'static str;

    fn entry_point() -> &'static str {
        "main"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry>;
}
