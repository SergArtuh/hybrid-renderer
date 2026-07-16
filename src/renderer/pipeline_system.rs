use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use notify::{Event, RecursiveMode, Watcher};

use crate::{
    core::{
        compute_task::{ComputeTaskTrait, ComputeTaskType},
        material::{Material, MaterialDefinition, MaterialTrait},
    },
    renderer::{
        RenderingEnvironment,
        materials::{HasDefinition, MaterialFactory},
    },
};

struct ShaderWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
    material_pipelines: Vec<MaterialDefinition>,
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
            material_pipelines: Vec::new(),
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
}

pub struct PipelineSystem {}

impl PipelineSystem {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn register_pipeline<'a, T: MaterialTrait>(render_env: &mut RenderingEnvironment)
    where
        T: HasDefinition,
    {
        let material_type = T::TYPE;
        let result = MaterialFactory::new(render_env).create_pipeline::<T>();

        if let Ok(result) = result {
            render_env
                .pipeline_resources
                .render_pipelines
                .insert(material_type, result.render_pipeline);

            render_env
                .pipeline_resources
                .pipeline_layouts
                .insert(material_type, result.pipeline_layout);

            render_env
                .pipeline_resources
                .materials
                .insert(material_type, result.bind_group_layout);

            // #[cfg(feature = "shader-hot-reload")]
            // {
            //     let reloader: ReloadFn = Box::new(move |_manager, env| pipeline_builder(env));
            //     self.reloaders.insert(material_type, reloader);
            // }
        }

        // #[cfg(feature = "shader-hot-reload")]
        // if let Some(w) = &mut self.shader_watcher {
        //     w.add_pipeline(pipeline_definition);
        // }
    }

    pub fn register_compute_pipeline<T: ComputeTaskTrait>(render_env: &mut RenderingEnvironment) {
        let base_compute_shaders_path = render_env
            .pipeline_resources
            .base_compute_shaders_path
            .clone();

        let task_type = T::TYPE;
        let shader_path = base_compute_shaders_path.join(T::get_shader_path());

        let bind_group_layout = render_env.render_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{:?} Layout", task_type)),
                entries: &T::get_bind_group_layout_entries(),
            },
        );

        let pipeline_layout = render_env.render_context.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{:?} Layout", task_type)),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        );

        let source = std::fs::read_to_string(shader_path.clone())
            .with_context(|| format!("Failed to read shader file at {:?}", shader_path.clone()));

        let source = match source {
            Ok(source) => source,
            Err(e) => panic!("Failed to read shader file at {:?}: {}", shader_path, e),
        };

        let label = shader_path.to_str().unwrap();

        let shader_module =
            render_env
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });

        let pipeline = render_env.render_context.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some(&format!("{:?} Pipeline", task_type)),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: T::entry_point(),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &std::collections::HashMap::new(),
                    zero_initialize_workgroup_memory: false,
                },
            },
        );

        render_env
            .pipeline_resources
            .compute_pipelines
            .insert(task_type, pipeline);
        render_env
            .pipeline_resources
            .compute_tasks
            .insert(task_type, Arc::new(bind_group_layout));
    }

    pub fn get_pipeline<'a>(
        render_env: &'a RenderingEnvironment,
        material: &Material,
    ) -> &'a wgpu::RenderPipeline {
        render_env
            .pipeline_resources
            .render_pipelines
            .get(&material.kind())
            .expect("Pipeline not registered for material type")
    }

    pub fn get_compute_pipeline<'a>(
        render_env: &'a RenderingEnvironment,
        task_type: ComputeTaskType,
    ) -> &'a wgpu::ComputePipeline {
        render_env
            .pipeline_resources
            .compute_pipelines
            .get(&task_type)
            .expect("Pipeline not registered for task type")
    }

    // #[allow(unused_variables)]
    // pub fn check_shader_updates(
    //     &mut self,
    //     context: &RenderContext,
    //     resources: &mut PipelineResources,
    // ) {
    //     #[cfg(feature = "shader-hot-reload")]
    //     {
    //         let modified_pipelines = if let Some(watcher) = &mut self.shader_watcher {
    //             watcher.process_events();
    //             watcher.get_modified_pipelines_definitions()
    //         } else {
    //             Vec::new()
    //         };

    //         for pipeline_definition in modified_pipelines {
    //             let mut pipeline_visitor_env = PipelineVisitorEnvironment {
    //                 pipeline_definition: &pipeline_definition,
    //                 context,
    //                 layout: interface,
    //             };

    //             let reloader = self
    //                 .reloaders
    //                 .get(&pipeline_definition.material_type)
    //                 .unwrap();

    //             let pipeline = reloader(self, &mut pipeline_visitor_env);

    //             if let Ok(pipeline) = pipeline {
    //                 self.render_pipelines
    //                     .insert(pipeline_definition.material_type, pipeline);
    //             } else {
    //                 println!(
    //                     "Failed to recompile pipeline for material type {:?}. Keeping old pipeline.",
    //                     pipeline_definition.material_type
    //                 );
    //             }

    //             let path = if let Ok(relative_path) = pipeline_definition
    //                 .shader_path
    //                 .strip_prefix(&self.base_shaders_path)
    //             {
    //                 relative_path
    //             } else {
    //                 &pipeline_definition.shader_path
    //             };
    //             println!(
    //                 "Shader modified {:?}, pipeline recompiled: {:?}",
    //                 path, pipeline_definition.material_type
    //             );
    //         }
    //     }
    // }
}
