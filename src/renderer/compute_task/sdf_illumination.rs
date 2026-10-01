use crate::{
    core::compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::SdfIllumination;

pub struct TaskDescriptor {
    pub input_normal_view: wgpu::TextureView,
    pub input_depth_view: wgpu::TextureView,
    pub output_ao_view: wgpu::TextureView,

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
        ]
    }

    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        // let cubemap_view = desc
        //     .input_normal_map
        //     .array_view
        //     .as_ref()
        //     .expect("Cubemap array view should be present");

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
                    ],
                    label: Some("Sdf Illumination Bind Group"),
                });

        let (width, height) = (desc.width, desc.height);
        let workgroup_size = 16;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
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
