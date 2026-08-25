#![allow(clippy::style, clippy::type_complexity)]

use crate::assets::asset_manager::AssetManager;
#[cfg(not(target_arch = "wasm32"))]
use crate::util::shader_watcher::ShaderWatcherSystem;
use std::collections::HashMap;
use std::sync::Arc;

use wgpu::{ExperimentalFeatures, SurfaceColorSpace};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};

use crate::input::camera_controller::CameraControllerSystem;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};

use crate::core::render_context::RenderContext;
use crate::renderer::{RendererSystem, RenderingEnvironment};
use crate::stage::Stage;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

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

pub struct AppContext<'a> {
    pub window: Arc<Window>,
    pub render_env: RenderingEnvironment<'a>,
}

pub trait Application: 'static + Sized {
    fn init(ctx: &mut AppContext<'_>) -> (Self, Stage);
    fn load_assets(&mut self, _asset_manager: &mut AssetManager<'_>) {}
    fn update(&mut self, _ctx: &mut AppContext<'_>, _stage: &mut Stage, _delta_time: f32) {}
}

pub struct App {}

fn touch_pinch_distance(touches: &HashMap<u64, PhysicalPosition<f64>>) -> f64 {
    let mut points = touches.values();
    match (points.next(), points.next()) {
        (Some(a), Some(b)) => {
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            (dx * dx + dy * dy).sqrt()
        }
        _ => 0.0,
    }
}

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
        event_loop.set_control_flow(ControlFlow::Poll);

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
            flags: wgpu::InstanceFlags::advanced_debugging(),
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
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

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

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            wgpu::PresentMode::Immediate
        } else {
            surface_caps.present_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
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

        let (mut app, mut stage) = T::init(&mut app_ctx);

        let mut asset_manager = AssetManager::new(&app_ctx.render_env);
        app.load_assets(&mut asset_manager);

        let mut last_time = Instant::now();
        let mut last_mouse_pos: Option<(f64, f64)> = None;
        let mut active_touches: HashMap<u64, PhysicalPosition<f64>> = HashMap::new();
        let mut last_pinch_distance: Option<f64> = None;

        let target_fps = 30.0;
        let target_frame_duration = std::time::Duration::from_secs_f64(1.0 / target_fps);
        let mut last_render_time = Instant::now();

        let event_handler = move |event: Event<()>, elwt: &EventLoopWindowTarget<()>| match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    },
                ..
            } => {
                if let Some(ref mut controller) = stage.camera_controller {
                    CameraControllerSystem::process_keyboard(controller, &key_event);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { button, state, .. },
                ..
            } => {
                if state == winit::event::ElementState::Released {
                    last_mouse_pos = None;
                }
                if let Some(ref mut controller) = stage.camera_controller {
                    CameraControllerSystem::process_mouse_button(controller, button, state);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if let Some((last_x, last_y)) = last_mouse_pos {
                    let delta_x = position.x - last_x;
                    let delta_y = position.y - last_y;

                    if let Some(ref mut controller) = stage.camera_controller {
                        CameraControllerSystem::process_mouse(controller, delta_x, delta_y);
                    }
                }
                last_mouse_pos = Some((position.x, position.y));
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                if let Some(ref mut controller) = stage.camera_controller {
                    CameraControllerSystem::process_scroll(controller, &delta);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Touch(touch),
                ..
            } => match touch.phase {
                TouchPhase::Started => {
                    active_touches.insert(touch.id, touch.location);
                    if let Some(ref mut controller) = stage.camera_controller {
                        match active_touches.len() {
                            1 => CameraControllerSystem::process_mouse_button(
                                controller,
                                MouseButton::Left,
                                ElementState::Pressed,
                            ),
                            2 => {
                                CameraControllerSystem::process_mouse_button(
                                    controller,
                                    MouseButton::Left,
                                    ElementState::Released,
                                );
                                last_pinch_distance = Some(touch_pinch_distance(&active_touches));
                            }
                            _ => {}
                        }
                    }
                }
                TouchPhase::Moved => {
                    let previous = active_touches.insert(touch.id, touch.location);
                    match active_touches.len() {
                        1 => {
                            if let Some(previous) = previous {
                                let delta_x = touch.location.x - previous.x;
                                let delta_y = touch.location.y - previous.y;
                                if let Some(ref mut controller) = stage.camera_controller {
                                    CameraControllerSystem::process_mouse(
                                        controller, delta_x, delta_y,
                                    );
                                }
                            }
                        }
                        2 => {
                            let distance = touch_pinch_distance(&active_touches);
                            if let Some(last_distance) = last_pinch_distance {
                                let pinch_delta = distance - last_distance;
                                if let Some(ref mut controller) = stage.camera_controller {
                                    CameraControllerSystem::process_pinch(
                                        controller,
                                        &MouseScrollDelta::PixelDelta(PhysicalPosition::new(
                                            0.0,
                                            pinch_delta,
                                        )),
                                    );
                                }
                            }
                            last_pinch_distance = Some(distance);
                        }
                        _ => {}
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    active_touches.remove(&touch.id);
                    if let Some(ref mut controller) = stage.camera_controller {
                        match active_touches.len() {
                            0 => {
                                CameraControllerSystem::process_mouse_button(
                                    controller,
                                    MouseButton::Left,
                                    ElementState::Released,
                                );
                                last_pinch_distance = None;
                            }
                            1 => {
                                CameraControllerSystem::process_mouse_button(
                                    controller,
                                    MouseButton::Left,
                                    ElementState::Pressed,
                                );
                                last_pinch_distance = None;
                            }
                            _ => {
                                last_pinch_distance = Some(touch_pinch_distance(&active_touches));
                            }
                        }
                    }
                }
            },
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

                AssetManager::update(&mut app_ctx.render_env, &mut stage);
                if let Some(ref mut controller) = stage.camera_controller {
                    CameraControllerSystem::update_camera(
                        controller,
                        &mut stage.main_camera,
                        delta_time,
                    )
                }
                app.update(&mut app_ctx, &mut stage, delta_time.as_secs_f32());
                let frame_data = stage.make_frame_data();
                RendererSystem::render(&mut app_ctx.render_env, &frame_data);

                #[cfg(not(target_arch = "wasm32"))]
                ShaderWatcherSystem::update(&mut app_ctx.render_env);
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let next_frame_time = last_render_time + target_frame_duration;

                if now >= next_frame_time {
                    window.request_redraw();

                    if now.duration_since(last_render_time) > target_frame_duration * 2 {
                        last_render_time = now;
                    } else {
                        last_render_time += target_frame_duration;
                    }
                } else {
                    elwt.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
                }
            }
            _ => (),
        };

        #[cfg(target_arch = "wasm32")]
        event_loop.spawn(event_handler);

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run(event_handler).unwrap();
    }
}
