use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use notify::{Event, RecursiveMode, Watcher};

use crate::{
    core::{
        material::{Material, MaterialTrait, MaterialType},
        render_context::RenderContext,
    },
    renderer::layout_interface::LayoutInterface,
};

use anyhow::Context;

type LayoutFetcher = fn() -> &'static [wgpu::VertexBufferLayout<'static>];

#[derive(Clone, Debug)]
struct PipelineDefinition {
    shader_path: PathBuf,
    layout_fetcher: LayoutFetcher,
    material_type: MaterialType,
}

struct ShaderWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
    pipelines: Vec<PipelineDefinition>,
    modified_shaders: HashSet<PathBuf>,
    last_tick: Instant,
}

impl ShaderWatcher {
    const TICK_INTERVAL: Duration = Duration::from_millis(500);
    fn new(shader_path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(shader_path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
            pipelines: Vec::new(),
            modified_shaders: HashSet::new(),
            last_tick: Instant::now(),
        })
    }

    pub fn process_events(&mut self) {
        if self.last_tick.elapsed() < Self::TICK_INTERVAL {
            return;
        }
        self.last_tick = Instant::now();

        for res in self.receiver.try_iter() {
            match res {
                Ok(event) => {
                    event.paths.iter().for_each(|path| {
                        if event.kind.is_modify() {
                            self.modified_shaders.insert(path.clone());
                        }
                    });
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    fn get_modified_pipelines_definitions(&mut self) -> Vec<PipelineDefinition> {
        let mut modified_pipelines = Vec::new();
        let modified_shaders = self.get_modified_shaders();
        for modified_shader in modified_shaders {
            for pipeline in self.pipelines.iter() {
                if pipeline.shader_path == modified_shader {
                    modified_pipelines.push(pipeline.clone());
                }
            }
        }
        modified_pipelines
    }

    fn get_modified_shaders(&mut self) -> Vec<PathBuf> {
        let modified_shaders = self.modified_shaders.iter().cloned().collect();
        self.modified_shaders.clear();
        modified_shaders
    }

    fn add_pipeline(&mut self, pipeline_definition: PipelineDefinition) {
        self.pipelines.push(pipeline_definition);
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
    ) {
        let material_type = T::TYPE;
        let full_shader_path = self.base_shaders_path.clone().join(shader_path);

        let layout_fetcher: LayoutFetcher = T::get_layout;

        let pipeline_definition = PipelineDefinition {
            layout_fetcher,
            shader_path: full_shader_path,
            material_type,
        };

        let render_pipeline = self.build_pipeline(&pipeline_definition, context, interface);

        if let Ok(render_pipeline) = render_pipeline {
            self.pipelines.insert(material_type, render_pipeline);
        }

        if let Some(w) = &mut self.shader_watcher {
            w.add_pipeline(pipeline_definition);
        }
    }

    fn build_pipeline(
        &mut self,
        pipeline_definition: &PipelineDefinition,
        context: &RenderContext,
        interface: &LayoutInterface,
    ) -> Result<wgpu::RenderPipeline, anyhow::Error> {
        context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_path = &pipeline_definition.shader_path;
        let source = std::fs::read_to_string(shader_path)
            .with_context(|| format!("Failed to read shader file at {:?}", shader_path))?;

        let label = shader_path
            .to_str()
            .context("Shader path contains invalid UTF-8 characters")?;

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
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
                        buffers: (pipeline_definition.layout_fetcher)(),
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

        self.modules.insert(
            pipeline_definition
                .shader_path
                .to_str()
                .unwrap()
                .to_string(),
            shader_module,
        );

        let maybe_error = pollster::block_on(context.device.pop_error_scope());

        if let Some(error) = maybe_error {
            println!(
                "Pipeline Validation Error: {:?}, {:?}",
                pipeline_definition.shader_path, error
            );
            return Err(anyhow::anyhow!("Pipeline Validation Error: {:?}", error));
        }

        Ok(render_pipeline)
    }

    pub fn get_pipeline(&self, material: &Material) -> &wgpu::RenderPipeline {
        self.pipelines
            .get(&material.kind())
            .expect("Pipeline not registered for material type")
    }

    pub fn check_shader_updates(&mut self, context: &RenderContext, interface: &LayoutInterface) {
        let modified_pipelines = if let Some(watcher) = &mut self.shader_watcher {
            watcher.process_events();
            watcher.get_modified_pipelines_definitions()
        } else {
            Vec::new()
        };

        for pipeline_definition in modified_pipelines {
            let pipeline = self.build_pipeline(&pipeline_definition, context, interface);

            if let Ok(pipeline) = pipeline {
                self.pipelines
                    .insert(pipeline_definition.material_type, pipeline);
            } else {
                println!(
                    "Failed to recompile pipeline for material type {:?}. Keeping old pipeline.",
                    pipeline_definition.material_type
                );
            }

            let path = if let Ok(relative_path) = pipeline_definition
                .shader_path
                .strip_prefix(&self.base_shaders_path)
            {
                relative_path
            } else {
                &pipeline_definition.shader_path
            };
            println!(
                "Shader modified {:?}, pipeline recompiled: {:?}",
                path, pipeline_definition.material_type
            );
        }
    }
}
