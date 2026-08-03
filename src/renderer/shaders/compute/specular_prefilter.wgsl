struct SpecularPrefilterUniform {
    config: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_cube<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(2) var input_sampler: sampler;
@group(0) @binding(3) var<uniform> specular_prefilter_uniform: SpecularPrefilterUniform;

const PI: f32 = 3.14159265359;

fn radical_inverse_vdc(bits_in: u32) -> f32 {
    var bits = bits_in;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10;
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(N), radical_inverse_vdc(i));
}

fn importance_sample_ggx(xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
    let a = roughness;
    
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y));
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
    
    let H = vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
    
    var up = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.z) < 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);
    
    return normalize(tangent * H.x + bitangent * H.y + N * H.z);
}

fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {
    let st = uv * 2.0 - 1.0;
    switch (face) {
        case 0u: { return vec3<f32>(1.0, -st.y, -st.x); }
        case 1u: { return vec3<f32>(-1.0, -st.y, st.x); }
        case 2u: { return vec3<f32>(st.x, 1.0, st.y); }
        case 3u: { return vec3<f32>(st.x, -1.0, -st.y); }
        case 4u: { return vec3<f32>(st.x, -st.y, 1.0); }
        case 5u: { return vec3<f32>(-st.x, -st.y, -1.0); }
        default: { return vec3<f32>(0.0); }
    }
}

fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let a = roughness;
    let a2 = a * a;
    let NdotH2 = NdotH * NdotH;
    let nom = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return nom / (PI * denom * denom);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let roughness = specular_prefilter_uniform.config.x;
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims.xy);
    let N = normalize(get_cube_direction(uv, id.z));
    
    let V = N;

    var prefiltered_color = vec3(0.0);
    var total_weight = 0.0;
    
    let num_samples = 512u;
    let input_width = f32(textureDimensions(input_tex).x);

    for (var i = 0u; i < num_samples; i++) {
        let xi = hammersley(i, num_samples);
        
        let H = importance_sample_ggx(xi, N, roughness);
        
        let L = normalize(2.0 * dot(V, H) * H - V);
        
        let NdotL = max(dot(N, L), 0.0);
        if (NdotL > 0.0) {
            let NdotH = max(dot(N, H), 0.0);
            let VdotH = max(dot(V, H), 0.0);
            
            let D = distribution_ggx(NdotH, roughness);
            let pdf = (D * NdotH / (4.0 * VdotH)) + 0.0001;
            
            let sa_texel = 4.0 * PI / (6.0 * input_width * input_width);
            let sa_sample = 1.0 / (f32(num_samples) * pdf + 0.0001);
            let lod = select(0.5 * log2(sa_sample / sa_texel), 0.0, roughness == 0.0);

            prefiltered_color += textureSampleLevel(input_tex, input_sampler, L, lod).rgb * NdotL;
            total_weight += NdotL;
        }
    }

    var color = prefiltered_color / max(total_weight, 0.0001);
    color = select(color, vec3(0.0), any(color != color));

    textureStore(output_tex, id.xy, id.z, vec4(color, 1.0));
}