use std::sync::Arc;

use anyhow::Context;

use crate::{
    core::{
        compute_task::{ComputeTaskTrait, ComputeTaskType},
        material::{Material, MaterialTrait, PipelineKey},
    },
    renderer::{
        RenderingEnvironment,
        materials::{MaterialFactory, MaterialHasDefinition},
    },
};

pub struct PipelineSystem {}

impl PipelineSystem {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn register_pipeline<T: MaterialTrait>(render_env: &mut RenderingEnvironment)
    where
        T: MaterialHasDefinition,
    {
        let material_type = T::TYPE;
        let result = MaterialFactory::new(render_env).create_pipeline::<T>();

        if let Ok(result) = result {
            for (key, pipeline) in result.render_pipelines {
                render_env
                    .pipeline_resources
                    .render_pipelines
                    .insert((material_type, key), pipeline);
            }

            render_env
                .pipeline_resources
                .pipeline_layouts
                .insert(material_type, result.pipeline_layout);

            render_env
                .pipeline_resources
                .materials
                .insert(material_type, result.bind_group_layout);

            let shader_path = render_env
                .pipeline_resources
                .base_shaders_path
                .clone()
                .join(T::get_shader_path());

            render_env
                .pipeline_resources
                .material_to_shader_registry
                .insert(material_type, shader_path);
        }
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

        render_env
            .pipeline_resources
            .compute_task_to_shader_registry
            .insert(task_type, shader_path);
    }

    pub fn get_default_pipeline<'a>(
        render_env: &'a RenderingEnvironment,
        material: &Material,
    ) -> Option<&'a wgpu::RenderPipeline> {
        Self::get_pipeline(render_env, material, PipelineKey::default())
    }

    pub fn get_pipeline<'a>(
        render_env: &'a RenderingEnvironment,
        material: &Material,
        key: PipelineKey,
    ) -> Option<&'a wgpu::RenderPipeline> {
        render_env
            .pipeline_resources
            .render_pipelines
            .get(&(material.kind(), key))
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
}
