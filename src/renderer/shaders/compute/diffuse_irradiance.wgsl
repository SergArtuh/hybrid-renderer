@group(0) @binding(0) var input_tex: texture_cube<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(2) var input_sampler: sampler;

const PI: f32 = 3.14159265359;


fn is_nan(val: f32) -> bool {
    return val != val;
}

fn sanitize_color(color: vec3<f32>) -> vec3<f32> {
    if (is_nan(color.r) || is_nan(color.g) || is_nan(color.b)) {
        return vec3(0.0); 
    }
    
    if (color.r > 65000.0 || color.g > 65000.0 || color.b > 65000.0) {
         return vec3(0.0); 
    }

    return color;
}


fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {
    let st = uv * 2.0 - 1.0;
    switch (face) {
        case 0u: { return vec3<f32>(1.0, -st.y, -st.x); }  // +X
        case 1u: { return vec3<f32>(-1.0, -st.y, st.x); }  // -X
        case 2u: { return vec3<f32>(st.x, 1.0, st.y); }    // +Y
        case 3u: { return vec3<f32>(st.x, -1.0, -st.y); }  // -Y
        case 4u: { return vec3<f32>(st.x, -st.y, 1.0); }   // +Z
        case 5u: { return vec3<f32>(-st.x, -st.y, -1.0); } // -Z
        default: { return vec3<f32>(0.0); }
    }
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3(p.xyx) * vec3(443.897, 441.423, 437.195));
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.xx + p3.yz) * p3.zy);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims.xy);
    let normal = normalize(get_cube_direction(uv, id.z));

    var up = vec3(0.0, 1.0, 0.0);
    if (abs(normal.y) > 0.999) { up = vec3(1.0, 0.0, 0.0); }
    let right = normalize(cross(up, normal));
    let forward = cross(normal, right);

    var irradiance = vec3(0.0);
    let num_samples = 64u; 

    for (var i = 0u; i < num_samples; i++) {
        let rand = hash22(uv + f32(i)); 

        let phi = 2.0 * PI * rand.x;
        let cos_theta = sqrt(1.0 - rand.y);
        let sin_theta = sqrt(rand.y);

        let local_sample = vec3(
            cos(phi) * sin_theta,
            sin(phi) * sin_theta,
            cos_theta
        );

        let sample_dir = local_sample.x * right + local_sample.y * forward + local_sample.z * normal;
        irradiance += textureSampleLevel(input_tex, input_sampler, sample_dir, 0.0).rgb;
    }

    let final_color = irradiance / f32(num_samples);
    let color = sanitize_color(final_color);

    textureStore(output_tex, id.xy, id.z, vec4(color, 1.0));
}