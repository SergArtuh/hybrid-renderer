@group(0) @binding(0) var output_tex: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) {
        return;
    }

    textureStore(output_tex, id.xy, id.z, vec4<f32>(0.0, 0.5, 0.0, 1.0));
}