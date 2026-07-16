use hybrid_renderer::assets::asset_manager::AssetManager;
use hybrid_renderer::assets::model::Model;
use hybrid_renderer::input::camera_controller::{
    OrbitCameraController, OrbitCameraControllerDescriptor,
};
use std::sync::Arc;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use hybrid_renderer::assets::camera::Camera;
use hybrid_renderer::core::math::Vec3;
use hybrid_renderer::core::render_context::RenderContext;
use hybrid_renderer::renderer::{RendererSystem, RenderingEnvironment};
use hybrid_renderer::stage::Stage;

//const CAMERA_DISTANCE: f32 = 250.0;
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

    let camera = Camera::new(Vec3::new(0.0, 1., CAMERA_DISTANCE))
        .with_target(Vec3::ZERO)
        .with_fov(60.0)
        .with_near(0.1)
        .with_far(1000.0)
        .with_aspect(size.width as f32 / size.height as f32);

    let mut stage = Stage::new(camera);

    // let mut camera_controller = FreeCameraController::new(Default::default());
    let mut camera_controller = OrbitCameraController::new(OrbitCameraControllerDescriptor {
        target: Vec3::ZERO,
        distance: CAMERA_DISTANCE,
        start_pitch: 0.15,
        start_yaw: 2.0 * std::f32::consts::PI / 3.0,
        ..Default::default()
    });

    let render_context = RenderContext::new(device, queue, surface, config);
    let mut render_env = RenderingEnvironment::create_and_initialize(render_context);

    let asset_manager = AssetManager::new(&render_env);
    let models = asset_manager
        //.load_gltf_models("assets/models/MetalRoughSpheres.glb")
        .load_gltf_models("assets/models/ClearCoatTest.glb")
        //.load_gltf_models("assets/models/NormalTangentTest.glb")
        //.load_gltf_models("assets/models/NormalTangentMirrorTest.glb")
        .unwrap();

    for model_node in &models.scene_roots {
        let mut model = Model::new(Arc::clone(model_node), glam::Mat4::IDENTITY);
        model.transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.6, 0.0));

        stage.add_model(model);
    }
    let skydome = asset_manager
        //.load_skydome("assets/modern_buildings_night_1k.exr")
        //.load_skydome("assets/modern_buildings_night_8k.exr", 150.0, 0.95)
        //.load_skydome("assets/zwartkops_curve_sunset_4k.exr", 150.0, 0.95)
        .load_skydome("assets/cannon_1k.hdr", 150.0, 0.95)
        .unwrap();
    stage.set_skydome(skydome);

    let mut last_time = Instant::now();
    //let mut frame_time = 0.0;
    event_loop
        .run(move |event, elwt| match event {
            winit::event::Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    },
                ..
            } => {
                camera_controller.process_keyboard(&key_event);
            }
            winit::event::Event::WindowEvent {
                event: WindowEvent::MouseInput { button, state, .. },
                ..
            } => {
                camera_controller.process_mouse_button(button, state);
            }
            winit::event::Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion { delta },
                ..
            } => {
                camera_controller.process_mouse(delta.0, delta.1);
            }
            winit::event::Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                camera_controller.process_scroll(&delta);
            }
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

                camera_controller.update_camera(&mut stage.main_camera, delta_time);

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
