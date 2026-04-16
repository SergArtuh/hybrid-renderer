// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
//     @location(2) uv: vec2<f32>,
// };

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)  normal: vec3<f32>,
    @location(1)  uv: vec2<f32>,
};

struct CameraUniform {
    proj_view: mat4x4<f32>,
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
};


@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@group(2) @binding(0) var t_texture: texture_2d<f32>;
@group(2) @binding(1) var s_texture: sampler;


@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
    //vertex: VertexInput
) -> VertexOutput {
    var pos: vec2<f32>;
    var uv: vec2<f32>;

    switch in_vertex_index {
        case 0u, 3u: { pos = vec2<f32>(-1.0,  1.0); uv = vec2<f32>(0.0, 0.0); }
        case 1u:     { pos = vec2<f32>( 1.0,  1.0); uv = vec2<f32>(1.0, 0.0); }
        case 2u, 4u: { pos = vec2<f32>( 1.0, -1.0); uv = vec2<f32>(1.0, 1.0); }
        case 5u:     { pos = vec2<f32>(-1.0, -1.0); uv = vec2<f32>(0.0, 1.0); }
        default:     { pos = vec2<f32>( 0.0,  0.0); uv = vec2<f32>(0.0, 0.0); }
    }

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_texture, in.uv);
}