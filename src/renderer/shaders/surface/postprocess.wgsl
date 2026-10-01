struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}


@group(0) @binding(0) var<uniform> camera: CameraUniform;


@group(2) @binding(0) var t_normal_buffer: texture_2d<f32>;
@group(2) @binding(1) var common_sampler: sampler;
@group(2) @binding(2) var t_depth_buffer: texture_depth_2d;


const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2(-1.0,  1.0), vec2( 1.0,  1.0), vec2( 1.0, -1.0),
        vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2(-1.0, -1.0)
    );


    var out: VertexOutput;
    let p = pos[in_vertex_index];
    let uv = vec2((p.x + 1.0) * 0.5, (1.0 - p.y) * 0.5);
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(t_normal_buffer, common_sampler, in.uv).rgb;
    let raw_depth = textureSample(t_depth_buffer, common_sampler, in.uv);
    let depth = pow(raw_depth, 15.0);
    let tone_mapped_rgb = hdr_color.rgb;
    let gamma_corrected = tone_mapped_rgb;

    //return vec4<f32>(gamma_corrected, 1.0-depth);
     return vec4<f32>(gamma_corrected, 0.7);
}