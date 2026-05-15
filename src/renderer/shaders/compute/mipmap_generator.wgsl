@group(0) @binding(0) var src_tex: texture_2d_array<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d_array<rgba16float, write>;


fn get_luminance(v: vec3<f32>) -> f32 {
    return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn karis_weight(v: vec3<f32>) -> f32 {
    let luma = get_luminance(v);
    return 1.0 / (1.0 + log(1.0 + luma));
}


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(dst_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) {
        return;
    }

    let src_coord = id.xy * 2u;
    let face = i32(id.z);
    

    let c00 = textureLoad(src_tex, src_coord + vec2(0u, 0u), face, 0);
    let c10 = textureLoad(src_tex, src_coord + vec2(1u, 0u), face, 0);
    let c01 = textureLoad(src_tex, src_coord + vec2(0u, 1u), face, 0);
    let c11 = textureLoad(src_tex, src_coord + vec2(1u, 1u), face, 0);

    let w00 = karis_weight(c00.rgb);
    let w10 = karis_weight(c10.rgb);
    let w01 = karis_weight(c01.rgb);
    let w11 = karis_weight(c11.rgb);

    let total_weight = w00 + w10 + w01 + w11;
    let color = (c00 * w00 + c10 * w10 + c01 * w01 + c11 * w11) / total_weight;

    let safe_color = clamp(color, vec4<f32>(0.0), vec4<f32>(65500.0));
    textureStore(dst_tex, id.xy, id.z, safe_color);
}