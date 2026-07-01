use std::sync::Arc;

use crate::{
    core::{
        material::{PhysicalMaterial, SkydomeEnvironmentMaterial, SpriteMaterial},
        render_context::RenderContext,
    },
    renderer::{
        compute_task::{
            ClearCubemapTask, DiffuseIrradianceTask, EquirectToCubemapTask, MipmapGeneratorTask,
            SpecularPrefilterTask,
        },
        frame_target::FrameTarget,
        layout_interface::{GlobalResources, LayoutInterface, ModelResources},
        materials::skydome_material::SkydomeMaterialDefinition,
        pipeline_manager::PipelineManager,
    },
    stage::frame_data::FrameData,
};

pub mod compute_task;
pub mod frame_target;
pub mod layout_interface;
pub mod materials;
pub mod passes;
pub mod pipeline_manager;

use materials::{PbrMaterialDefinition, SpriteMaterialDefinition};
use passes::forward::ForwardPass;

pub struct RenderingEnvironment<'a> {
    pub render_context: RenderContext<'a>,
    pub layout_interface: LayoutInterface,
    pub pipeline_manager: PipelineManager,
}

impl<'a> RenderingEnvironment<'a> {
    pub fn new(render_context: RenderContext<'a>) -> Self {
        let layout_interface = LayoutInterface::new(&render_context);
        let pipeline_manager = PipelineManager::new();
        Self {
            render_context,
            layout_interface,
            pipeline_manager,
        }
    }
}

pub struct Renderer {
    forward_pass: ForwardPass,
    //pipeline_manager: PipelineManager,
    //pub layout_interface: Arc<RefCell<LayoutInterface>>,
    global_resources: GlobalResources,
    model_resources: ModelResources,
    depth_texture_view: wgpu::TextureView,
}

impl Renderer {
    pub fn new(rendering_environment: &mut RenderingEnvironment) -> Self {
        let render_context = &rendering_environment.render_context;
        let layout_interface = &mut rendering_environment.layout_interface;
        let pipeline_manager = &mut rendering_environment.pipeline_manager;
        render_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        //let layout_interface = Arc::new(RefCell::new(LayoutInterface::new(render_context)));
        let global_resources = GlobalResources::new(render_context, &layout_interface.global);
        let model_resources = ModelResources::new(render_context, &layout_interface.model);
        let depth_texture_view = Self::create_depth_texture(render_context);

        //let mut pipeline_manager = PipelineManager::new();
        pipeline_manager.register_pipeline::<PhysicalMaterial>(
            render_context,
            layout_interface,
            "pbr_material.wgsl",
            PbrMaterialDefinition::create_pipeline,
        );
        pipeline_manager.register_pipeline::<SpriteMaterial>(
            render_context,
            layout_interface,
            "sprite_material.wgsl",
            SpriteMaterialDefinition::create_pipeline,
        );
        pipeline_manager.register_pipeline::<SkydomeEnvironmentMaterial>(
            render_context,
            layout_interface,
            "skydome.wgsl",
            SkydomeMaterialDefinition::create_pipeline,
        );

        pipeline_manager
            .register_compute_pipeline::<EquirectToCubemapTask>(render_context, layout_interface);

        pipeline_manager
            .register_compute_pipeline::<ClearCubemapTask>(render_context, layout_interface);

        pipeline_manager
            .register_compute_pipeline::<DiffuseIrradianceTask>(render_context, layout_interface);

        pipeline_manager
            .register_compute_pipeline::<MipmapGeneratorTask>(render_context, layout_interface);

        pipeline_manager
            .register_compute_pipeline::<SpecularPrefilterTask>(render_context, layout_interface);

        let forward_pass = ForwardPass::default();

        Self {
            forward_pass,
            global_resources,
            model_resources,
            //pipeline_manager,
            //layout_interface,
            depth_texture_view,
        }
    }

    pub fn render(
        &mut self,
        rendering_environment: &mut RenderingEnvironment,
        frame_data: &FrameData,
    ) {
        let render_context = &rendering_environment.render_context;
        let layout_interface = &mut rendering_environment.layout_interface;
        let pipeline_manager = &mut rendering_environment.pipeline_manager;

        let mut frame_target = self.begin_frame(
            render_context,
            layout_interface,
            pipeline_manager,
            frame_data,
        );

        {
            let mut render_pass =
                self.create_render_pass(&mut frame_target.encoder, &frame_target.view);

            self.forward_pass.execute(
                &mut render_pass,
                pipeline_manager,
                &self.global_resources,
                &self.model_resources,
                frame_data,
            );
        }

        self.end_frame(render_context, frame_target);
    }

    fn begin_frame(
        &mut self,
        render_context: &RenderContext,
        layout_interface: &mut LayoutInterface,
        pipeline_manager: &mut PipelineManager,
        frame_data: &FrameData,
    ) -> FrameTarget {
        let surface_texture = match render_context.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to get current texture: {:?}", e);
                panic!("Failed to get current texture");
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        pipeline_manager.check_shader_updates(render_context, layout_interface);

        self.global_resources
            .update_camera_uniform_buffer(render_context, &frame_data.camera_uniform);

        frame_data
            .get_environment_texture()
            .map(|environment_texture| {
                self.global_resources.update_skybox_texture(
                    render_context,
                    &layout_interface.global,
                    Arc::clone(&environment_texture.skybox),
                    Arc::clone(&environment_texture.irradiance),
                    Arc::clone(&environment_texture.specular),
                )
            });

        self.model_resources
            .update_buffer(render_context, &frame_data.render_items);

        FrameTarget {
            surface_texture,
            view,
            encoder,
        }
    }

    fn end_frame(&self, render_context: &RenderContext, target: FrameTarget) {
        render_context
            .queue
            .submit(std::iter::once(target.encoder.finish()));
        //render_context.device.poll(wgpu::Maintain::Wait);
        target.surface_texture.present();
    }

    fn create_depth_texture(render_context: &RenderContext) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: render_context.config.width,
            height: render_context.config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PipelineManager::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = render_context.device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_render_pass<'a>(
        self: &'a Self,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass
    }
}
