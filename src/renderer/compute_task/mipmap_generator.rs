use std::sync::Arc;

use crate::{
    core::{
        compute_task::{ComputeTaskInstance, ComputeTaskTrait, ComputeTaskType},
        texture::Texture,
        texture_builder::TextureBuilder,
    },
    renderer::{RenderingEnvironment, compute_task::ComputeTaskFactory},
};

const TASK_TYPE: ComputeTaskType = ComputeTaskType::MipmapGenerator;

pub struct TaskDescriptor {
    pub source_texture: Arc<Texture>,
    pub output_texture: Arc<Texture>,
}
pub struct Task {
    pub bind_group: wgpu::BindGroup,
    pub dispatch_size: (u32, u32, u32),
}

impl ComputeTaskTrait for Task {
    const TYPE: ComputeTaskType = TASK_TYPE;

    fn get_shader_path() -> &'static str {
        "mipmap_generator.wgsl"
    }

    fn get_bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ]
    }

    fn create_instance(
        render_env: &RenderingEnvironment,
        desc: Self::Descriptor,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<ComputeTaskInstance, anyhow::Error> {
        let current_mip_view =
            desc.source_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Source Cube View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                });

        let target_mip_view =
            desc.output_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Scratch Storage View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                });

        let bind_group =
            render_env
                .render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&current_mip_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&target_mip_view),
                        },
                    ],
                    label: Some("Clear Cubemap Bind Group"),
                });

        let mip_width = desc.output_texture.width;
        let mip_height = desc.output_texture.height;
        let workgroup_size = 8;

        Ok(ComputeTaskInstance {
            task_type: Self::TYPE,
            bind_group,
            dispatch_size: (
                (mip_width + workgroup_size - 1) / workgroup_size,
                (mip_height + workgroup_size - 1) / workgroup_size,
                6,
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

    pub fn process(&self, source_texture: &Arc<Texture>) -> Result<Arc<Texture>, anyhow::Error> {
        let origin_width = source_texture.width;
        let origin_height = source_texture.height;

        let mip_count = (origin_width.max(origin_height) as f32).log2().floor() as u32 + 1;

        let result_texture = Arc::new(
            TextureBuilder::new(
                &self.render_env.render_context.device,
                &self.render_env.render_context.queue,
            )
            .with_label("final_mipmapped_cubemap")
            .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
            .with_size(origin_width, origin_height)
            .with_mip_level_count(mip_count)
            .as_cubemap()
            .with_usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .build(),
        );

        let mut encoder = self
            .render_env
            .render_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Initial Mip Copy"),
            });

        encoder.copy_texture_to_texture(
            source_texture.texture.as_image_copy(),
            result_texture.texture.as_image_copy(),
            wgpu::Extent3d {
                width: origin_width,
                height: origin_height,
                depth_or_array_layers: 6,
            },
        );
        self.render_env
            .render_context
            .queue
            .submit(Some(encoder.finish()));

        let mut current_src = Arc::clone(source_texture);

        for target_mip in 1..mip_count {
            let mip_width = (origin_width >> target_mip).max(1);
            let mip_height = (origin_height >> target_mip).max(1);

            println!("Generating mip {} {}x{}", target_mip, mip_width, mip_height);

            let temp_mip_out = Arc::new(
                TextureBuilder::new(
                    &self.render_env.render_context.device,
                    &self.render_env.render_context.queue,
                )
                .with_label(&format!("temp_mip_{}", target_mip))
                .with_wgpu_format(wgpu::TextureFormat::Rgba16Float)
                .with_size(mip_width, mip_height)
                .as_cubemap()
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                )
                .build(),
            );

            let compute_task_factory = ComputeTaskFactory::new(self.render_env);
            let mipmap_task = compute_task_factory.create_task::<Task>(TaskDescriptor {
                source_texture: Arc::clone(&current_src),
                output_texture: Arc::clone(&temp_mip_out),
            });

            compute_task_factory
                .create_executor()
                .record(&mipmap_task)
                .execute()
                .wait();

            let mut copy_encoder = self
                .render_env
                .render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(&format!("Copy Mip {}", target_mip)),
                });

            copy_encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &temp_mip_out.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &result_texture.texture,
                    mip_level: target_mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 6,
                },
            );

            self.render_env
                .render_context
                .queue
                .submit(Some(copy_encoder.finish()));
            current_src = temp_mip_out;
        }

        self.render_env
            .render_context
            .device
            .poll(wgpu::Maintain::Wait);
        Ok(result_texture)
    }
}
