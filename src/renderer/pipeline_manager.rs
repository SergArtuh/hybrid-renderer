use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use notify::{Event, RecursiveMode, Watcher};

use crate::{
    core::{
        compute_task::{ComputeTaskTrait, ComputeTaskType},
        material::{Material, MaterialTrait, MaterialType},
        render_context::RenderContext,
    },
    renderer::{layout_interface::LayoutInterface, materials::PipelineVisitorEnvironment},
};

type LayoutFetcher = fn() -> &'static [wgpu::VertexBufferLayout<'static>];

#[derive(Clone, Debug)]
pub(crate) struct PipelineDefinition {
    pub shader_path: PathBuf,
    pub layout_fetcher: LayoutFetcher,
    pub material_type: MaterialType,
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

pub type PipelineBuilderFn =
    fn(&PipelineVisitorEnvironment) -> Result<wgpu::RenderPipeline, anyhow::Error>;

type ReloadFn = Box<
    dyn Fn(
        &PipelineManager,
        &PipelineVisitorEnvironment,
    ) -> Result<wgpu::RenderPipeline, anyhow::Error>,
>;

pub struct PipelineManager {
    render_pipelines: HashMap<MaterialType, wgpu::RenderPipeline>,
    compute_pipelines: HashMap<ComputeTaskType, wgpu::ComputePipeline>,
    base_shaders_path: PathBuf,
    base_compute_shaders_path: PathBuf,
    #[cfg(feature = "shader-hot-reload")]
    shader_watcher: Option<ShaderWatcher>,
    #[cfg(feature = "shader-hot-reload")]
    reloaders: HashMap<MaterialType, ReloadFn>,
}

impl PipelineManager {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn new() -> Self {
        let base_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders")
            .join("surface");

        let base_compute_shaders_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("renderer")
            .join("shaders")
            .join("compute");

        #[cfg(feature = "shader-hot-reload")]
        let shader_watcher = ShaderWatcher::new(&base_shaders_path);
        #[cfg(feature = "shader-hot-reload")]
        let shader_watcher = match shader_watcher {
            Ok(watcher) => Some(watcher),
            Err(_) => None,
        };

        Self {
            render_pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            base_shaders_path,
            base_compute_shaders_path,
            #[cfg(feature = "shader-hot-reload")]
            shader_watcher,
            #[cfg(feature = "shader-hot-reload")]
            reloaders: HashMap::new(),
        }
    }

    pub fn register_pipeline<T: MaterialTrait>(
        &mut self,
        context: &RenderContext,
        interface: Arc<RefCell<LayoutInterface>>,
        shader_path: &str,
        pipeline_builder: PipelineBuilderFn,
    ) {
        let material_type = T::TYPE;
        let full_shader_path = self.base_shaders_path.clone().join(shader_path);

        let layout_fetcher: LayoutFetcher = T::get_layout;

        let pipeline_definition = PipelineDefinition {
            layout_fetcher,
            shader_path: full_shader_path,
            material_type,
        };

        let pipeline_visitor_env = PipelineVisitorEnvironment {
            pipeline_definition: &pipeline_definition,
            context,
            layout: interface,
        };

        let render_pipeline = pipeline_builder(&pipeline_visitor_env);

        if let Ok(render_pipeline) = render_pipeline {
            self.render_pipelines.insert(material_type, render_pipeline);

            #[cfg(feature = "shader-hot-reload")]
            {
                let reloader: ReloadFn = Box::new(move |_manager, env| pipeline_builder(env));
                self.reloaders.insert(material_type, reloader);
            }
        }

        #[cfg(feature = "shader-hot-reload")]
        if let Some(w) = &mut self.shader_watcher {
            w.add_pipeline(pipeline_definition);
        }
    }

    pub fn register_compute_pipeline<T: ComputeTaskTrait>(
        &mut self,
        context: &RenderContext,
        interface: Arc<RefCell<LayoutInterface>>,
    ) {
        let task_type = T::TYPE;
        let shader_path = self.base_compute_shaders_path.join(T::get_shader_path());

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{:?} Layout", task_type)),
                    entries: &T::get_bind_group_layout_entries(),
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{:?} Layout", task_type)),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let source = std::fs::read_to_string(shader_path.clone())
            .with_context(|| format!("Failed to read shader file at {:?}", shader_path.clone()));

        let source = match source {
            Ok(source) => source,
            Err(e) => panic!("Failed to read shader file at {:?}: {}", shader_path, e),
        };

        let label = shader_path.to_str().unwrap();

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(&format!("{:?} Pipeline", task_type)),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: T::entry_point(),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &std::collections::HashMap::new(),
                    zero_initialize_workgroup_memory: false,
                },
            });

        self.compute_pipelines.insert(task_type, pipeline);
        interface
            .borrow_mut()
            .compute_tasks
            .insert(task_type, Arc::new(bind_group_layout));
    }

    pub fn get_pipeline(&self, material: &Material) -> &wgpu::RenderPipeline {
        self.render_pipelines
            .get(&material.kind())
            .expect("Pipeline not registered for material type")
    }

    pub fn get_compute_pipeline(&self, task_type: ComputeTaskType) -> &wgpu::ComputePipeline {
        self.compute_pipelines
            .get(&task_type)
            .expect("Pipeline not registered for task type")
    }

    #[allow(unused_variables)]
    pub fn check_shader_updates(
        &mut self,
        context: &RenderContext,
        interface: Arc<RefCell<LayoutInterface>>,
    ) {
        #[cfg(feature = "shader-hot-reload")]
        {
            let modified_pipelines = if let Some(watcher) = &mut self.shader_watcher {
                watcher.process_events();
                watcher.get_modified_pipelines_definitions()
            } else {
                Vec::new()
            };

            for pipeline_definition in modified_pipelines {
                let pipeline_visitor_env = PipelineVisitorEnvironment {
                    pipeline_definition: &pipeline_definition,
                    context,
                    layout: Arc::clone(&interface),
                };

                let reloader = self
                    .reloaders
                    .get(&pipeline_definition.material_type)
                    .unwrap();

                let pipeline = reloader(self, &pipeline_visitor_env);

                if let Ok(pipeline) = pipeline {
                    self.render_pipelines
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
}
