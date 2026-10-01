@group(0) @binding(0) var input_normal_tex: texture_2d<f32>;
@group(0) @binding(1) var input_depth_tex: texture_depth_2d;
@group(0) @binding(2) var output_ao_tex: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var nearest_sampler: sampler;



@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_ao_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    textureStore(output_ao_tex, id.xy, vec4<f32>(0.5, 1.0, 0.0, 0.0));
}