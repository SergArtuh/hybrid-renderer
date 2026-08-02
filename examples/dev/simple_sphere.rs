use hybrid_renderer::assets::model::Model;
use hybrid_renderer::core::model_node::ModelNode;
use hybrid_renderer::renderer::materials::MaterialFactory;
use hybrid_renderer::util::shader_watcher::ShaderWatcherSystem;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use hybrid_renderer::assets::camera::Camera;
use hybrid_renderer::core::math::Vec3;
use hybrid_renderer::core::mesh::Mesh;
use hybrid_renderer::core::render_context::RenderContext;
use hybrid_renderer::renderer::{RendererSystem, RenderingEnvironment};
use hybrid_renderer::stage::Stage;
use hybrid_renderer::util::geometry_generator::MeshUtil;

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
    println!("GPU: {}, API: {:?}", info.name, info.backend);

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

    let render_context = RenderContext::new(device, queue, surface, config);
    let mut render_env = RenderingEnvironment::create_and_initialize(render_context);

    let camera = Camera::new(Vec3::new(0.0, 0.0, CAMERA_DISTANCE))
        .with_target(Vec3::new(0.0, 0.0, 0.0))
        .with_fov(45.0)
        .with_aspect(size.width as f32 / size.height as f32);

    let mesh_data = MeshUtil::new_cube(0.8);
    let mesh = Mesh::from_data(&render_env.render_context.device, &mesh_data);

    let material = MaterialFactory::new(&render_env).create_default_material();
    let model_node = ModelNode::new(mesh, material);
    let model = Model::new(Arc::new(model_node), glam::Mat4::IDENTITY);

    let mut stage = Stage::new(camera);
    stage.add_model(model);

    let mut last_time = Instant::now();
    let mut frame_time = 0.0;
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

                let camera_x = f32::sin(frame_time) * CAMERA_DISTANCE;
                let camera_z = f32::cos(frame_time) * CAMERA_DISTANCE;

                stage.main_camera.position.x = camera_x;
                stage.main_camera.position.z = camera_z;
                let frame_data = stage.make_frame_data();
                RendererSystem::render(&mut render_env, &frame_data);
                ShaderWatcherSystem::update(&mut render_env);
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
