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
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@group(2) @binding(0) var t_base_color: texture_2d<f32>;
@group(2) @binding(1) var t_normal: texture_2d<f32>;
@group(2) @binding(2) var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(3) var t_occlusion: texture_2d<f32>;
@group(2) @binding(4) var t_emissive: texture_2d<f32>;
@group(2) @binding(5) var common_sampler: sampler;


@vertex
fn vs_main(
    //@builtin(vertex_index) in_vertex_index: u32
    vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    //out.clip_position = camera.proj_view * (model.model_matrix * vec4<f32>(vertex.position, 1.0) +  vec4<f32>(0.0, 0.6, 0.0, 0.0));
    out.clip_position = camera.proj_view * model.model_matrix * vec4<f32>(vertex.position, 1.0);
    out.normal = vertex.normal;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color = textureSample(t_base_color, common_sampler, in.uv);
    var normal = textureSample(t_normal, common_sampler, in.uv);
    var metallic_roughness = textureSample(t_metallic_roughness, common_sampler, in.uv);
    var occlusion = textureSample(t_occlusion, common_sampler, in.uv);
    var emissive = textureSample(t_emissive, common_sampler, in.uv);

    let orm = textureSample(t_metallic_roughness, common_sampler, in.uv);
    let roughness = orm.g;
    let metallic = orm.b;
    
    return base_color;
    //return vec4<f32>(in.clip_position.xyz / in.clip_position.w, 1.0);
    //return vec4<f32>(roughness, roughness, roughness, 1.0);
    //return vec4<f32>(metallic, metallic, metallic, 1.0);
    //return vec4<f32>(occlusion.rgb, 1.0);
    //return vec4<f32>(emissive.rgb, 1.0);
}