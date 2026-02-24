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

struct CameraUniform {
    proj_view: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    //@builtin(vertex_index) in_vertex_index: u32
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.proj_view * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.normal, 1.0);
}