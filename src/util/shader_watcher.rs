#[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::Duration,
    time::Instant,
};

#[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
use notify::{Event, RecursiveMode, Watcher};

#[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
use crate::core::{compute_task::ComputeTaskType, material::MaterialType};

use crate::core::compute_task::ComputeTaskTrait;
use crate::core::material::MaterialTrait;
use crate::renderer::{
    RenderingEnvironment, materials::MaterialHasDefinition, pipeline_resources::PipelineResources,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::renderer::pipeline_system::PipelineSystem;

#[allow(dead_code)]
mod shader_watcher_stub {
    #[cfg(target_arch = "wasm32")]
    use super::*;
    #[cfg(target_arch = "wasm32")]
    pub struct ShaderWatcherResources;

    #[cfg(target_arch = "wasm32")]
    impl ShaderWatcherResources {
        pub fn create_and_initialize(_pipeline_resources: &PipelineResources) -> Self {
            Self {}
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub struct ShaderWatcherSystem;

    #[allow(unused_variables)]
    #[cfg(target_arch = "wasm32")]
    impl ShaderWatcherSystem {
        pub fn register_pipeline<T: MaterialTrait>(_render_env: &mut RenderingEnvironment)
        where
            T: MaterialHasDefinition,
        {
        }

        pub fn register_compute_pipeline<T: ComputeTaskTrait>(
            _render_env: &mut RenderingEnvironment,
        ) {
        }

        pub fn update(_render_env: &mut RenderingEnvironment) {}
    }
}

mod shader_watcher_hot_reload {
    #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
    use super::*;
    pub struct ShaderWatcherResources {
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        _watcher: notify::RecommendedWatcher,
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        modified_shaders: HashSet<PathBuf>,
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        last_tick: Instant,
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        material_rebuild_functions: HashMap<MaterialType, fn(&mut RenderingEnvironment)>,
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        compute_task_rebuild_functions: HashMap<ComputeTaskType, fn(&mut RenderingEnvironment)>,
    }

    impl ShaderWatcherResources {
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        const TICK_INTERVAL: Duration = Duration::from_millis(500);

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
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
        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
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

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        pub fn register_compute_pipeline<T: ComputeTaskTrait>(
            render_env: &mut RenderingEnvironment,
        ) {
            render_env
                .shader_watcher_resources
                .as_mut()
                .unwrap()
                .compute_task_rebuild_functions
                .insert(T::TYPE, |env| {
                    PipelineSystem::register_compute_pipeline::<T>(env)
                });
        }

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
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

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
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

        #[cfg(all(feature = "shader-hot-reload", not(target_arch = "wasm32")))]
        fn get_modified_shaders(resources: &mut ShaderWatcherResources) -> Vec<PathBuf> {
            let modified_shaders = resources.modified_shaders.iter().cloned().collect();
            resources.modified_shaders.clear();
            modified_shaders
        }
    }
}

#[cfg(not(feature = "shader-hot-reload"))]
use shader_watcher_stub as shader_watcher;

#[cfg(feature = "shader-hot-reload")]
use shader_watcher_hot_reload as shader_watcher;

pub use shader_watcher::{ShaderWatcherResources, ShaderWatcherSystem};
