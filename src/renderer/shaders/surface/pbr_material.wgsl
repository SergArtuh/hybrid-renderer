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
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
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

struct MaterialData {
    base_color_factor: vec4<f32>,
    emissive_and_scale: vec4<f32>, 
    pbr_factors: vec4<f32>, 
    clearcoat_factors: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var env_cubemap: texture_cube<f32>;
@group(0) @binding(2) var s_texture: sampler;
@group(0) @binding(3) var t_irradiance: texture_cube<f32>;
@group(0) @binding(4) var t_specular: texture_cube<f32>;

@group(1) @binding(0) var<uniform> model: ModelUniform;

@group(2) @binding(0) var t_base_color: texture_2d<f32>;
@group(2) @binding(1) var t_normal: texture_2d<f32>;
@group(2) @binding(2) var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(3) var t_occlusion: texture_2d<f32>;
@group(2) @binding(4) var t_emissive: texture_2d<f32>;
@group(2) @binding(5) var common_sampler: sampler;
@group(2) @binding(6) var<uniform> material_data: MaterialData;
@group(2) @binding(7) var t_clearcoat: texture_2d<f32>;
@group(2) @binding(8) var t_clearcoat_roughness: texture_2d<f32>;
@group(2) @binding(9) var t_clearcoat_normal: texture_2d<f32>;

const PI: f32 = 3.14159265359;
const PI_INV: f32 = 1.0 / PI;
const F0_DIELECTRIC = vec3<f32>(0.04);
const F0_CLEARCOAT = 0.04;

const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;


fn processed_env_brdf_approx(specular_color: vec3<f32>, roughness: f32, NoV: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    let AB = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return specular_color * AB.x + AB.y;
}

@vertex
fn vs_main(
    vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model_matrix * vec4<f32>(vertex.position, 1.0);

    let normal_w = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    let tangent_w = normalize((model.model_matrix * vec4<f32>(vertex.tangent.xyz, 0.0)).xyz);
    let bitangent_w = normalize(cross(normal_w, tangent_w)) * vertex.tangent.w; 

    out.world_position = world_pos.xyz;
    out.clip_position = camera.proj_view * world_pos;
    out.normal = normal_w;
    out.tangent = tangent_w;
    out.bitangent = bitangent_w;

    out.uv = vertex.uv;
    return out;
}

fn processed_fresnel_schlick_roughness(VoN: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let fresnel_power = pow(clamp(1.0 - VoN, 0.0, 1.0), 5.0);
    let F90 = max(vec3<f32>(1.0 - roughness), F0); 
    return F0 + (F90 - F0) * fresnel_power;
}

fn processed_diffuse(N: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let albedo = textureSample(t_base_color, common_sampler, uv).rgb;
    let diffuse_color = textureSample(t_irradiance, s_texture, N).rgb;
    let diffuse_color_factor = material_data.base_color_factor.rgb;
    return diffuse_color * diffuse_color_factor * albedo;
}

fn processed_specular(R: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let orm = textureSample(t_metallic_roughness, common_sampler, uv);
    let roughness_sqrt = orm.g;
    let roughness_factor = material_data.pbr_factors.x;
    let roughness = roughness_sqrt * roughness_factor;

    let specular_mip_count = f32(textureNumLevels(t_specular) - 1u);
    let specular_lod = roughness * specular_mip_count;
    let hdr_color_specular = textureSampleLevel(t_specular, s_texture, R, specular_lod).rgb;
    return hdr_color_specular;
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    let base_color = textureSample(t_base_color, common_sampler, in.uv).rgb * material_data.base_color_factor.rgb;
    let orm = textureSample(t_metallic_roughness, common_sampler, in.uv);
    let emissive_color = textureSample(t_emissive, common_sampler, in.uv).rgb;
    
    let roughness_sqrt = orm.g;
    let roughness_factor = material_data.pbr_factors.x;
    let metallic_factor = material_data.pbr_factors.y;
    let occlusion_strength = material_data.pbr_factors.b;
    let emissive_factor = material_data.emissive_and_scale.rgb;
    let cc_factor = textureSample(t_clearcoat, common_sampler, in.uv).r * material_data.pbr_factors.a;
    let cc_roughness = textureSample(t_clearcoat_roughness, common_sampler, in.uv).r * material_data.clearcoat_factors.r;

    let roughness = roughness_sqrt * roughness_factor;
    let metalness  = orm.b * metallic_factor;

    let normal_map = textureSample(t_normal, common_sampler, in.uv).rgb;
    let normal_texture_space = normalize(normal_map*2.0 -1.0 );

    var N = normalize(in.normal);
    let T = normalize(in.tangent);
    let B = normalize(in.bitangent);

    let TBN = mat3x3<f32>(T, B, N);
    N = normalize(TBN * normal_texture_space);

    let V = normalize(camera.position.xyz - in.world_position);
    let R = normalize(reflect(-V, N));

    let diffuse_color = processed_diffuse(N, in.uv);
    let specular_color = processed_specular(R, in.uv);

    let VoN = max(dot(V, N), 0.0);
    let F0 = mix(vec3<f32>(0.04), base_color, metalness);

    let c0 = 1.0 - roughness;
    let c1 = pow(1.0 - VoN, 5.0);
    let env_brdf = processed_env_brdf_approx(F0, roughness, VoN);

    let f0 = mix(vec3<f32>(0.04), base_color, metalness);
    let F = processed_fresnel_schlick_roughness(VoN, f0, roughness);
	let kS = env_brdf;
	let kD = vec3(1.0f, 1.0f, 1.0f) - kS;


    let color = emissive_color * emissive_factor + diffuse_color * kD + specular_color * kS;

    
    let gamma_corrected = pow(color, vec3(INV_GAMMA));
    return vec4<f32>(color, 1.0);
}