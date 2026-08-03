@group(0) @binding(0) var input_tex: texture_cube<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(2) var input_sampler: sampler;

const PI: f32 = 3.14159265359;

fn radical_inverse_vdc(bits_in: u32) -> f32 {
    var bits = bits_in;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(N), radical_inverse_vdc(i));
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

fn compute_lod(pdf: f32, num_samples: f32, width: f32) -> f32 {
    return 0.5 * log2(6.0 * width * width / (num_samples * pdf));
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims.xy);
    let normal = normalize(get_cube_direction(uv, id.z));

    var up = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(normal.z) < 0.999);
    let right = normalize(cross(up, normal));
    let forward = cross(normal, right);

    var irradiance = vec3(0.0);
    let num_samples = 128u;
    let input_width = f32(textureDimensions(input_tex).x);

    for (var i = 0u; i < num_samples; i++) {
        let xi = hammersley(i, num_samples);
        
        let phi = 2.0 * PI * xi.x;
        let cos_theta = sqrt(1.0 - xi.y);
        let sin_theta = sqrt(xi.y);

        let local_sample = vec3(
            cos(phi) * sin_theta,
            sin(phi) * sin_theta,
            cos_theta
        );

        let sample_dir = normalize(local_sample.x * right + local_sample.y * forward + local_sample.z * normal);
        
        let pdf = max(cos_theta / PI, 0.001); 
        let lod = compute_lod(pdf, f32(num_samples), input_width);

        irradiance += textureSampleLevel(input_tex, input_sampler, sample_dir, lod).rgb;
    }

    let color = irradiance / f32(num_samples);
    
    let safe_color = clamp(color, vec3<f32>(0.0), vec3<f32>(65500.0));

    textureStore(output_tex, id.xy, id.z, vec4(safe_color, 1.0));
}