use wgpu::util::DeviceExt;

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
        uniforms::SDFUniform,
    },
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::SdfIllumination;

pub struct TaskDescriptor {
    pub input_normal_view: wgpu::TextureView,
    pub input_depth_view: wgpu::TextureView,
    pub output_ao_view: wgpu::TextureView,
    pub global_bounding_group: wgpu::BindGroup,
    pub sdf_texture_view: wgpu::TextureView,
    pub sdf_uniform: SDFUniform,

    pub width: u32,
    pub height: u32,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        "sdf_illumination.wgsl"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<SDFUniform>() as u64
                    ),
                },
                count: None,
            },
        ]
    }

    fn use_global_bindings() -> bool {
        true
    }

    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let sdf_uniform_buffer = render_env.render_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sdf Uniform Buffer"),
                contents: bytemuck::cast_slice(&[desc.sdf_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group =
            render_env
                .render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&desc.input_normal_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&desc.input_depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&desc.output_ao_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(
                                &render_env.render_resources.common_nearest_sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(&desc.sdf_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &sdf_uniform_buffer,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<SDFUniform>() as u64
                                ),
                            }),
                        },
                    ],
                    label: Some("Sdf Illumination Bind Group"),
                });

        let (width, height) = (desc.width, desc.height);
        let workgroup_size = 16;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group: vec![(0, desc.global_bounding_group), (1, bind_group)],
            dispatch_size: (
                (width + workgroup_size - 1) / workgroup_size,
                (height + workgroup_size - 1) / workgroup_size,
                1,
            ),
        })
    }

    type Descriptor = TaskDescriptor;
}

pub struct Provider<'a> {
    render_env: &'a RenderingEnvironment<'a>,
}

impl<'a> Provider<'a> {
    pub fn new(render_env: &'a RenderingEnvironment<'a>) -> Self {
        Self { render_env }
    }

    pub fn process(
        &self,
        input_normal_view: &wgpu::TextureView,
        input_depth_view: &wgpu::TextureView,
        output_ao_view: &wgpu::TextureView,
        sdf_texture_view: &wgpu::TextureView,
        sdf_uniform: &SDFUniform,
        width: u32,
        height: u32,
    ) -> Result<(), anyhow::Error> {
        let compute_task_factory = ComputeTaskFactory::new(self.render_env);
        // let result_texture = Arc::new(
        //     TextureBuilder::new(
        //         &self.render_env.render_context.device,
        //         &self.render_env.render_context.queue,
        //     )
        //     .with_label("diffuse_irradiance")
        //     .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
        //     .with_size(irradiance_resolution, irradiance_resolution)
        //     .as_cubemap()
        //     .with_usage(
        //         wgpu::TextureUsages::TEXTURE_BINDING
        //             | wgpu::TextureUsages::STORAGE_BINDING
        //             | wgpu::TextureUsages::COPY_DST,
        //     )
        //     .build(),
        // );
        let sdf_task = compute_task_factory.create_task::<Task>(TaskDescriptor {
            input_normal_view: input_normal_view.clone(),
            input_depth_view: input_depth_view.clone(),
            output_ao_view: output_ao_view.clone(),
            sdf_texture_view: sdf_texture_view.clone(),
            sdf_uniform: sdf_uniform.clone(),
            global_bounding_group: self
                .render_env
                .render_resources
                .global_resources
                .bind_group
                .clone(),
            width,
            height,
        });

        compute_task_factory
            .create_executor()
            .record(&sdf_task)
            .execute()
            .wait();
        Ok(())
    }
}
