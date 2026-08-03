use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::{Duration, Instant},
};

use notify::{Event, RecursiveMode, Watcher};

use crate::{
    core::{
        compute_task::{ComputeTaskTrait, ComputeTaskType},
        material::{MaterialTrait, MaterialType},
    },
    renderer::{
        RenderingEnvironment, materials::MaterialHasDefinition,
        pipeline_resources::PipelineResources, pipeline_system::PipelineSystem,
    },
};

pub struct ShaderWatcherResources {
    _watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
    modified_shaders: HashSet<PathBuf>,
    last_tick: Instant,
    material_rebuild_functions: HashMap<MaterialType, fn(&mut RenderingEnvironment)>,
    compute_task_rebuild_functions: HashMap<ComputeTaskType, fn(&mut RenderingEnvironment)>,
}

impl ShaderWatcherResources {
    const TICK_INTERVAL: Duration = Duration::from_millis(500);
    pub fn create_and_initialize(pipeline_resources: &PipelineResources) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        watcher
            .watch(
                &pipeline_resources.base_shaders_path,
                RecursiveMode::Recursive,
            )
            .ok();

        watcher
            .watch(
                &pipeline_resources.base_compute_shaders_path,
                RecursiveMode::Recursive,
            )
            .ok();

        Self {
            _watcher: watcher,
            receiver: rx,
            modified_shaders: HashSet::new(),
            last_tick: Instant::now(),
            material_rebuild_functions: HashMap::new(),
            compute_task_rebuild_functions: HashMap::new(),
        }
    }
}

pub struct ShaderWatcherSystem;

impl ShaderWatcherSystem {
    pub fn register_pipeline<T: MaterialTrait>(render_env: &mut RenderingEnvironment)
    where
        T: MaterialHasDefinition,
    {
        render_env
            .shader_watcher_resources
            .as_mut()
            .unwrap()
            .material_rebuild_functions
            .insert(T::TYPE, |env| PipelineSystem::register_pipeline::<T>(env));
    }

    pub fn register_compute_pipeline<T: ComputeTaskTrait>(render_env: &mut RenderingEnvironment) {
        render_env
            .shader_watcher_resources
            .as_mut()
            .unwrap()
            .compute_task_rebuild_functions
            .insert(T::TYPE, |env| {
                PipelineSystem::register_compute_pipeline::<T>(env)
            });
    }

    fn process_events(resources: &mut ShaderWatcherResources) {
        if resources.last_tick.elapsed() < ShaderWatcherResources::TICK_INTERVAL {
            return;
        }
        resources.last_tick = Instant::now();

        for res in resources.receiver.try_iter() {
            match res {
                Ok(event) => {
                    event.paths.iter().for_each(|path| {
                        if event.kind.is_modify() {
                            resources.modified_shaders.insert(path.clone());
                        }
                    });
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    pub fn update(render_env: &mut RenderingEnvironment) {
        if let Some(resources) = render_env.shader_watcher_resources.as_mut() {
            Self::process_events(resources);
            let mut modified_matterals = Vec::new();
            let mut modified_compute_tasks = Vec::new();

            {
                let modified_shaders = Self::get_modified_shaders(
                    render_env.shader_watcher_resources.as_mut().unwrap(),
                );

                let material_to_shader_registry =
                    &render_env.pipeline_resources.material_to_shader_registry;

                for (material_type, shader_path) in material_to_shader_registry {
                    if modified_shaders.contains(&shader_path) {
                        modified_matterals.push(*material_type);
                    }
                }

                let compute_task_to_shader_registry = &render_env
                    .pipeline_resources
                    .compute_task_to_shader_registry;

                for (compute_task_type, shader_path) in compute_task_to_shader_registry {
                    if modified_shaders.contains(&shader_path) {
                        modified_compute_tasks.push(*compute_task_type);
                    }
                }
            }

            for material_type in modified_matterals {
                let rebuild_func = render_env
                    .shader_watcher_resources
                    .as_ref()
                    .unwrap()
                    .material_rebuild_functions
                    .get(&material_type)
                    .unwrap();

                println!("Reloading material pipeline: {:?} ", material_type);
                rebuild_func(render_env);
            }

            for compute_task_type in modified_compute_tasks {
                let rebuild_func = render_env
                    .shader_watcher_resources
                    .as_ref()
                    .unwrap()
                    .compute_task_rebuild_functions
                    .get(&compute_task_type)
                    .unwrap();

                println!("Reloading compute pipeline: {:?} ", compute_task_type);
                rebuild_func(render_env);
            }
        }
    }

    fn get_modified_shaders(resources: &mut ShaderWatcherResources) -> Vec<PathBuf> {
        let modified_shaders = resources.modified_shaders.iter().cloned().collect();
        resources.modified_shaders.clear();
        modified_shaders
    }
}
