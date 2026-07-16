use glam::Vec3;
use hybrid_renderer::{
    assets::{asset_manager::AssetManager, camera::Camera, model::Model},
    core::{
        material::Material, mesh::Mesh, model_node::ModelNode, render_context::RenderContext,
        texture_builder::TextureBuilder,
    },
    renderer::{
        RendererSystem, RenderingEnvironment,
        materials::{MaterialFactory, SpriteMaterialDescriptor},
    },
    stage::Stage,
    util::geometry_generator::MeshUtil,
};
use std::sync::Arc;
use std::time::Instant;
use std::{fs::File, io::Read};
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

const CAMERA_DISTANCE: f32 = 5.0;

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("wgpu Minimal")
            .build(&event_loop)
            .unwrap(),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        // backends: wgpu::Backends::DX12,
        flags: wgpu::InstanceFlags::from_build_config()
            | wgpu::InstanceFlags::VALIDATION
            | wgpu::InstanceFlags::GPU_BASED_VALIDATION,
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    println!("{:#?}", surface_caps);

    let ray_query = adapter.features().contains(wgpu::Features::RAY_QUERY);
    println!("Ray query: {}", ray_query);

    let info = adapter.get_info();
    println!("GPU: {}, Backend: {:?}", info.name, info.backend);

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("My Primary Device"),

            required_features: wgpu::Features::POLYGON_MODE_LINE,

            required_limits: wgpu::Limits {
                max_bind_groups: 4,
                ..wgpu::Limits::default()
            },
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();

    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let mut file = File::open("assets/flame.jpg").expect("Файл не найден!");
    let mut diffuse_bytes = Vec::<u8>::new();
    file.read_to_end(&mut diffuse_bytes).unwrap();

    let render_context = RenderContext::new(device, queue, surface, config);
    let mut render_env = RenderingEnvironment::create_and_initialize(render_context);

    let sprite_texture = Arc::new(
        TextureBuilder::new(
            &render_env.render_context.device,
            &render_env.render_context.queue,
        )
        .from_bytes(&diffuse_bytes)
        .with_filter(wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
        .build(),
    );

    let sprite_material =
        MaterialFactory::new(&render_env).create_material(SpriteMaterialDescriptor {
            texture: sprite_texture,
            grid_size: (8, 8),
        });

    let mesh_data = MeshUtil::new_procedural_quad();
    let mesh = Mesh::from_data(&render_env.render_context.device, &mesh_data);
    let model_node = Arc::new(ModelNode::new(
        mesh,
        hybrid_renderer::core::material::Material::Sprite(sprite_material),
    ));
    let model = Model::new(model_node.clone(), glam::Mat4::IDENTITY);

    let asset_manager = AssetManager::new(&render_env);
    let skydome = asset_manager
        .load_skydome("assets/modern_buildings_night_1k.exr", 150.0, 0.95)
        .unwrap();

    let camera = Camera::new(Vec3::new(0.0, 0.0, CAMERA_DISTANCE))
        .with_target(Vec3::new(0.0, 0.0, 0.0))
        .with_fov(45.0)
        .with_near(0.1)
        .with_far(1000.0)
        .with_aspect(size.width as f32 / size.height as f32);

    let mut stage = Stage::new(camera);
    stage.add_model(model);
    stage.set_skydome(skydome);

    let mut frame_time = 0.0;
    let sprite_animation_fps = 25.0;
    let sprite_animation_duration = 1.0 / sprite_animation_fps;
    let mut frame_count = 0;
    let mut last_time = Instant::now();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => elwt.exit(),
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_time);
                last_time = now;

                frame_time += delta_time.as_secs_f32();

                if frame_time >= sprite_animation_duration {
                    frame_time = 0.0;
                    frame_count += 1;
                    frame_count %= 8 * 8;
                }

                if let Some(material) = model_node.material.as_deref() {
                    if let Material::Sprite(sprite) = material {
                        sprite.set_frame(frame_count, &render_env.render_context.queue);
                    }
                }

                let frame_data = stage.make_frame_data();
                RendererSystem::render(&mut render_env, &frame_data);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        })
        .unwrap();
}

fn main() {
    env_logger::init();
    run();
}
