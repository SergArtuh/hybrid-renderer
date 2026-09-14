struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var env_cubemap: texture_cube<f32>;
@group(0) @binding(2) var s_texture: sampler;
@group(0) @binding(3) var t_irradiance: texture_cube<f32>;
@group(0) @binding(4) var t_specular: texture_cube<f32>;

@group(1) @binding(0) var<uniform> model: ModelUniform;



@vertex
fn vs_main(
    vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model_matrix * vec4<f32>(vertex.position, 1.0);
    let normal_w = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);

    out.world_position = world_pos.xyz;
    out.clip_position = camera.proj_view * world_pos;
    out.normal = normal_w;
    return out;
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    return vec4<f32>(in.normal, 1.0);
}