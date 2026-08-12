#[cfg(not(target_arch = "wasm32"))]
use crate::util::shader_watcher::ShaderWatcherSystem;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use wgpu::{ExperimentalFeatures, SurfaceColorSpace};
use winit::event::{Event, WindowEvent};

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::core::render_context::RenderContext;
use crate::renderer::{RendererSystem, RenderingEnvironment};
use crate::stage::Stage;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[macro_export]
macro_rules! main_app {
    ($app_type:ty, $title:expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        fn main() {
            $crate::app::App::run::<$app_type>($title);
        }

        #[cfg(target_arch = "wasm32")]
        #[$crate::reexport::wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn start() {
            $crate::app::App::run::<$app_type>($title);
        }

        #[cfg(target_arch = "wasm32")]
        fn main() {}
    };
}

const CAMERA_DISTANCE: f32 = 5.0;
pub struct AppContext<'a> {
    pub window: Arc<Window>,
    pub render_env: RenderingEnvironment<'a>,
}

pub trait Application: 'static + Sized {
    fn init(ctx: &mut AppContext<'_>) -> (Self, Stage);
    fn update(&mut self, _ctx: &mut AppContext<'_>, _delta_time: f32) {}
}

pub struct App {}

impl App {
    pub fn run<T: Application>(title: &'static str) {
        #[cfg(target_arch = "wasm32")]
        {
            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Info).expect("failed to initialize logger");
            wasm_bindgen_futures::spawn_local(Self::run_async::<T>(title));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();
            pollster::block_on(Self::run_async::<T>(title));
        }
    }

    #[allow(unused_variables)]
    async fn run_async<T: Application>(title: &'static str) {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        let event_loop = EventLoop::new().unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

        #[cfg(target_arch = "wasm32")]
        let window = {
            use winit::platform::web::WindowBuilderExtWebSys;
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document
                .get_element_by_id("wgpu-canvas")
                .expect("failed to find #wgpu-canvas in HTML")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            Arc::new(
                WindowBuilder::new()
                    .with_title(title)
                    .with_canvas(Some(canvas))
                    .build(&event_loop)
                    .expect("failed to create window"),
            )
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        #[cfg(target_arch = "wasm32")]
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .expect("no suitable graphics adapter found - does this browser support WebGPU?");

        #[cfg(not(target_arch = "wasm32"))]
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: true,
        }))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // #[cfg(not(target_arch = "wasm32"))]
        // let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        //     label: Some("My Primary Device"),

        //     required_features: wgpu::Features::POLYGON_MODE_LINE,

        //     required_limits: wgpu::Limits {
        //         max_bind_groups: 4,
        //         ..wgpu::Limits::default()
        //     },
        //     experimental_features: ExperimentalFeatures::default(),
        //     memory_hints: wgpu::MemoryHints::default(),
        //     trace: wgpu::Trace::Off,
        // }))
        // .unwrap();

        #[cfg(target_arch = "wasm32")]
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Web Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_bind_groups: 4,
                    ..adapter.limits()
                },
                experimental_features: ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to create device");

        #[cfg(not(target_arch = "wasm32"))]
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Primary Device"),

            required_features: wgpu::Features::POLYGON_MODE_LINE,

            required_limits: wgpu::Limits {
                max_bind_groups: 4,
                ..wgpu::Limits::default()
            },
            experimental_features: ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let size = window.inner_size();

        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .filter(|f| f.is_srgb())
        //     .next()
        //     .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &config);

        let render_context = RenderContext::new(device, queue, surface, config);
        let render_env = RenderingEnvironment::create_and_initialize(render_context);

        let mut app_ctx = AppContext {
            window: window.clone(),
            render_env,
        };

        #[allow(unused_mut)]
        let (mut app, mut stage) = T::init(&mut app_ctx);

        #[cfg(target_arch = "wasm32")]
        let mut last_time = web_time::Instant::now();

        #[cfg(not(target_arch = "wasm32"))]
        let mut last_time = std::time::Instant::now();

        let mut frame_time = 0.0f32;
        //let mut frame_count: u64 = 0;

        #[cfg(target_arch = "wasm32")]
        event_loop.spawn(move |event, _elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id,
            } if window_id == window.id() => {
                log::info!("resized to {}x{}", new_size.width, new_size.height);
                app_ctx.render_env.resize(new_size.width, new_size.height);
                if new_size.height > 0 {
                    stage.main_camera.aspect = new_size.width as f32 / new_size.height as f32;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                #[cfg(not(target_arch = "wasm32"))]
                let now = Instant::now();

                #[cfg(target_arch = "wasm32")]
                let now = web_time::Instant::now();

                let delta_time = now.duration_since(last_time);
                last_time = now;
                frame_time += delta_time.as_secs_f32();

                let camera_x = f32::sin(frame_time) * CAMERA_DISTANCE;
                let camera_z = f32::cos(frame_time) * CAMERA_DISTANCE;

                stage.main_camera.position.x = camera_x;
                stage.main_camera.position.z = camera_z;

                let frame_data = stage.make_frame_data();
                RendererSystem::render(&mut app_ctx.render_env, &frame_data);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        });

        #[cfg(not(target_arch = "wasm32"))]
        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => elwt.exit(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    window_id,
                } if window_id == window.id() => {
                    app_ctx.render_env.resize(new_size.width, new_size.height);
                    if new_size.height > 0 {
                        stage.main_camera.aspect = new_size.width as f32 / new_size.height as f32;
                    }
                }
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

                    app.update(&mut app_ctx, delta_time.as_secs_f32());

                    stage.main_camera.position.x = camera_x;
                    stage.main_camera.position.z = camera_z;
                    let frame_data = stage.make_frame_data();
                    RendererSystem::render(&mut app_ctx.render_env, &frame_data);
                    ShaderWatcherSystem::update(&mut app_ctx.render_env);
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            })
            .unwrap();
    }
}
