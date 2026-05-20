use crate::core::{
    compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
    render_context::RenderContext,
    texture::Texture,
};
use std::sync::Arc;

const TASK_TYPE: ComputeTaskType = ComputeTaskType::SpecularPrefilter;
const SHADER_PATH: &str = "specular_prefilter.wgsl";

pub struct TaskDescriptor {
    pub input_cubemap: Arc<Texture>,
    pub output_cubemap: Arc<Texture>,
    pub config: Arc<wgpu::Buffer>, // TODO: make wraper
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        SHADER_PATH
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float, // TODO: make it configurable
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }

    fn create_instance(
        render_context: &RenderContext,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let cubemap_output_view = desc
            .output_cubemap
            .array_view
            .as_ref()
            .expect("Output view missing");

        // let config = SpecularPrefilterUniform::new(1.0); // TODO: make it configurable

        // let uniform_buffer =
        //     render_context
        //         .device
        //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //             label: Some("Specular Prefilter Uniform Buffer"),
        //             contents: bytemuck::cast_slice(&[config]),
        //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //         });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&desc.input_cubemap.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&cubemap_output_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&render_context.common_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(
                            desc.config.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: Some("Specular Prefilter Bind Group"),
            });

        let (width, height) = (desc.output_cubemap.width, desc.output_cubemap.height);
        let workgroup_size = 16;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
            dispatch_size: (
                (width + workgroup_size - 1) / workgroup_size,
                (height + workgroup_size - 1) / workgroup_size,
                6,
            ),
        })
    }

    type Descriptor = TaskDescriptor;
}
