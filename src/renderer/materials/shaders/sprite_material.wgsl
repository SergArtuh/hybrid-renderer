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

struct SpriteSheet {
    columns: f32,
    rows: f32,
    frame_index: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@group(2) @binding(0) var t_texture: texture_2d<f32>;
@group(2) @binding(1) var s_texture: sampler;
@group(2) @binding(2) var<uniform> atlas: SpriteSheet;


//@group(0) @binding(0) var<uniform> camera: CameraUniform;
//@group(1) @binding(0) var<uniform> model: ModelUniform;



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

    let scale = vec2<f32>(1.0 / atlas.columns, 1.0 / atlas.rows);
    let column = atlas.frame_index % atlas.columns;
    let row = floor(atlas.frame_index / atlas.columns);
    let offset = vec2<f32>(column * scale.x, row * scale.y);
    out.uv = offset + uv * scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_texture, in.uv);
}