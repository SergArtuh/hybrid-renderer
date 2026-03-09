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

struct ModelUniform {
    model_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@vertex
fn vs_main(
    //@builtin(vertex_index) in_vertex_index: u32
    vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.proj_view * model.model_matrix * vec4<f32>(vertex.position, 1.0);
    out.normal = vertex.normal;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.normal, 1.0);
}