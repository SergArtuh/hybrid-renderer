use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use notify::{Event, RecursiveMode, Watcher};

use crate::{
    core::{
        material::{Material, MaterialTrait, MaterialType},
        render_context::RenderContext,
    },
    renderer::layout_interface::LayoutInterface,
};

struct ShaderWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
}

impl ShaderWatcher {
    fn new(shader_path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(shader_path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }
}

pub struct PipelineManager {
    modules: HashMap<String, wgpu::ShaderModule>,
    pipelines: HashMap<MaterialType, wgpu::RenderPipeline>,
    base_shaders_path: PathBuf,
    shader_watcher: Option<ShaderWatcher>,
}

impl PipelineManager {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn new() -> Self {
        let base_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders");
        // TODO: add debug mode
        let shader_watcher = ShaderWatcher::new(&base_shaders_path);
        let shader_watcher = match shader_watcher {
            Ok(watcher) => Some(watcher),
            Err(_) => None,
        };

        Self {
            modules: HashMap::new(),
            pipelines: HashMap::new(),
            base_shaders_path,
            shader_watcher,
        }
    }

    pub fn register_pipeline<T: MaterialTrait>(
        &mut self,
        context: &RenderContext,
        interface: &LayoutInterface,
        shader_path: &str,
        vertex_layouts: &[wgpu::VertexBufferLayout],
    ) {
        let material_type = T::TYPE;
        let full_shader_path = self.base_shaders_path.clone().join(shader_path);
        let shader_module = self
            .modules
            .entry(full_shader_path.to_str().unwrap().to_string())
            .or_insert_with(|| {
                let source = std::fs::read_to_string(full_shader_path).expect("Shader not found");
                context
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(shader_path),
                        source: wgpu::ShaderSource::Wgsl(source.into()),
                    })
            });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&interface.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: vertex_layouts,
                        compilation_options: Default::default(),
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Self::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        self.pipelines.insert(material_type, render_pipeline);
    }

    pub fn get_pipeline(&self, material: &Material) -> &wgpu::RenderPipeline {
        self.pipelines
            .get(&material.kind())
            .expect("Pipeline not registered for material type")
    }

    pub fn check_shader_updates(&self) {
        if let Some(watcher) = &self.shader_watcher {
            for res in watcher.receiver.try_iter() {
                match res {
                    Ok(event) => println!("event: {:?}", event),
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        }
    }

    // fn shader_watcher(shader_path: &Path) -> Result<()> {
    //     let (tx, rx) = std::sync::mpsc::channel();
    //     let mut watcher = notify::recommended_watcher(tx)?;
    //     watcher.watch(shader_path, RecursiveMode::Recursive)?;

    //     for res in rx {
    //         match res {
    //             Ok(event) => println!("event: {:?}", event),
    //             Err(e) => println!("watch error: {:?}", e),
    //         }
    //     }

    //     Ok(())
    // }
}
