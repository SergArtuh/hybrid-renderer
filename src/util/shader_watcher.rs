use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use notify::{Event, RecursiveMode, Watcher};

struct ShaderWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
    //pipelines: Vec<PipelineMaterialDefinition>,
    modified_shaders: HashSet<PathBuf>,
    last_tick: Instant,
}

impl ShaderWatcher {
    /*
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

    pub fn update(&mut self, render_env: &RenderingEnvironment) {
        {
            self.process_events();
            let modified_pipelines = self.get_modified_pipelines_definitions();

            for pipeline_definition in modified_pipelines {
                let mut pipeline_visitor_env = PipelineVisitorEnvironment {
                    pipeline_definition: &pipeline_definition,
                    context: &render_env.render_context,
                    layout: &render_env.interface,
                };

                let reloader = self
                    .reloaders
                    .get(&pipeline_definition.material_type)
                    .unwrap();

                let pipeline = reloader(self, &mut pipeline_visitor_env);

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

    fn get_modified_pipelines_definitions(&mut self) -> Vec<PipelineMaterialDefinition> {
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

    // fn add_pipeline(&mut self, pipeline_definition: PipelineMaterialDefinition) {
    //     self.pipelines.push(pipeline_definition);
    // }

    */
}
