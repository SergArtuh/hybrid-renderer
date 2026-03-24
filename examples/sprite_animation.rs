use hybrid_renderer::core::material::SpriteMaterial;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

const SHADER_SOURCE: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)  normal: vec3<f32>,
    @location(1)  uv: vec2<f32>,
};

struct SpriteSheet {
    columns: f32,
    rows: f32,
    frame_index: f32,
    padding: f32,
};

@group(0) @binding(2) var<uniform> atlas: SpriteSheet;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var pos = array<vec2<f32>, 4>(
        vec2<f32>(-1.0,  1.0), // 0: Top-Left
        vec2<f32>( 1.0,  1.0), // 1: Top-Right
        vec2<f32>( 1.0, -1.0), // 2: Bottom-Right
        vec2<f32>(-1.0, -1.0)  // 3: Bottom-Left
    );

    var uv_coords = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0), // 0
        vec2<f32>(1.0, 0.0), // 1
        vec2<f32>(1.0, 1.0), // 2
        vec2<f32>(0.0, 1.0)  // 3
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    //out.normal = model.normal;

    let scale = vec2<f32>(1.0 / atlas.columns, 1.0 / atlas.rows);
    let column = atlas.frame_index % atlas.columns;
    let row = floor(atlas.frame_index / atlas.columns);
    let offset = vec2<f32>(column * scale.x, row * scale.y);
    out.uv = offset + uv_coords[in_vertex_index] * scale;
    return out;
}

@group(0) @binding(0) var t_texture: texture_2d<f32>;
@group(0) @binding(1) var s_texture: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_texture, in.uv);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

fn create_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    render_pass
}

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
    println!(
        "Использую видеокарту: {}, API: {:?}",
        info.name, info.backend
    );

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

    #[allow(unused_variables)]
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Simple Triangle Shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let mut file = File::open("assets/flame.jpg").expect("Файл не найден!");
    let mut diffuse_bytes = Vec::<u8>::new();
    file.read_to_end(&mut diffuse_bytes).unwrap();

    let mut sprite_material = SpriteMaterial::new(&diffuse_bytes, 8, 8, &device, &queue);

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&sprite_material.texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sprite_material.texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(
                    sprite_material.uniform_buffer.as_entire_buffer_binding(),
                ),
            },
        ],
        label: Some("texture_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

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
                let output = match surface.get_current_texture() {
                    Ok(output) => output,
                    Err(wgpu::SurfaceError::Lost) => {
                        surface.configure(&device, &config);
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        elwt.exit();
                        return;
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return;
                    }
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let now = Instant::now();
                    let delta_time = now.duration_since(last_time);
                    last_time = now;

                    frame_time += delta_time.as_secs_f32();

                    if frame_time >= sprite_animation_duration {
                        frame_time = 0.0;
                        frame_count += 1;
                        frame_count %= 8 * 8;
                    }

                    println!("Duration: {:.4}s", frame_time);

                    sprite_material.set_frame(frame_count, &queue);

                    let mut render_pass = create_render_pass(&mut encoder, &view);
                    render_pass.set_pipeline(&render_pipeline);

                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.set_bind_group(0, &texture_bind_group, &[]);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));

                output.present();
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
