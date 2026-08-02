use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

use hybrid_renderer::core::{
    material_builder::SpriteMaterialBuilder, render_context::RenderContext,
    texture::DefaultTextures, texture_builder::TextureBuilder,
};

const SHADER_SOURCE: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

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
    //@builtin(vertex_index) in_vertex_index: u32
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.uv = model.uv;
    return out;
}

@group(0) @binding(0) var t_texture: texture_2d<f32>;
@group(0) @binding(1) var s_texture: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(t_texture, s_texture, in.uv);

    // Simplified Reinhard tone mapping
    let tone_mapped_rgb = hdr_color.rgb / (hdr_color.rgb + vec3<f32>(1.0));
    //return vec4<f32>(tone_mapped_rgb, hdr_color.a);
    return vec4<f32>(hdr_color.rgb, 1.0);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        // position: Location 0
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        // normal: Location 1
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3,
        },
        // uv: Location 2
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x2,
        },
    ],
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 0.0],
    },
];

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

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let render_context = RenderContext::new(device, queue, surface, config);

    //let mut file = File::open("assets/modern_buildings_night_1k.exr").expect("Файл не найден!");
    let mut file = File::open("assets/flame.jpg").expect("Файл не найден!");
    let mut diffuse_bytes = Vec::<u8>::new();
    file.read_to_end(&mut diffuse_bytes).unwrap();

    let sprite_texture = Arc::new(
        TextureBuilder::new(&render_context.device, &render_context.queue)
            .from_bytes(&diffuse_bytes)
            .with_filter(wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
            .build(),
    );

    let texture_bind_group_layout =
        render_context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
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

    let default_textures = DefaultTextures::new(&render_context.device, &render_context.queue);

    let common_nearest_sampler = render_context
        .device
        .create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

    let sprite_material = SpriteMaterialBuilder::new()
        .with_texture(Arc::clone(&sprite_texture))
        .with_grid_size(1, 1)
        .build(
            &render_context,
            &texture_bind_group_layout,
            &default_textures,
            &common_nearest_sampler,
        );
    //let sprite_material = SpriteMaterial::new(sprite_texture.view, 1, 1, &device);

    // let texture_bind_group = render_context
    //     .device
    //     .create_bind_group(&wgpu::BindGroupDescriptor {
    //         layout: &texture_bind_group_layout,
    //         entries: &[
    //             wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: wgpu::BindingResource::TextureView(&sprite_material.texture),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 1,
    //                 resource: wgpu::BindingResource::Sampler(&sprite_texture.sampler),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 2,
    //                 resource: wgpu::BindingResource::Buffer(
    //                     sprite_material.uniform_buffer.as_entire_buffer_binding(),
    //                 ),
    //             },
    //         ],
    //         label: Some("texture_bind_group"),
    //     });

    let pipeline_layout =
        render_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

    let render_pipeline =
        render_context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[VERTEX_LAYOUT],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: render_context.config.format,
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
                let output = match render_context.surface.get_current_texture() {
                    Ok(output) => output,
                    Err(wgpu::SurfaceError::Lost) => {
                        render_context
                            .surface
                            .configure(&render_context.device, &render_context.config);
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

                let mut encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                {
                    let mut render_pass = create_render_pass(&mut encoder, &view);
                    render_pass.set_pipeline(&render_pipeline);

                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.set_bind_group(0, &sprite_material.bind_group, &[]);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }

                render_context
                    .queue
                    .submit(std::iter::once(encoder.finish()));

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
