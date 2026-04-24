@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d_array<rgba32float, write>;

const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;
const INV_PI: f32 = 0.31830988618;
const INV_TWO_PI: f32 = 0.15915494309;

fn get_direction_from_cubemap(coord: vec2<f32>, face: u32, size: vec2<f32>) -> vec3<f32> {
    let uv = 2.0 * (coord + 0.5) / size - 1.0;
    var dir: vec3<f32>;
    switch (face) {
        case 0u: { dir = vec3<f32>( 1.0, -uv.y, -uv.x); } // +X
        case 1u: { dir = vec3<f32>(-1.0, -uv.y,  uv.x); } // -X
        case 2u: { dir = vec3<f32>( uv.x,  1.0,  uv.y); } // +Y
        case 3u: { dir = vec3<f32>( uv.x, -1.0, -uv.y); } // -Y
        case 4u: { dir = vec3<f32>( uv.x, -uv.y,  1.0); } // +Z
        case 5u: { dir = vec3<f32>(-uv.x, -uv.y, -1.0); } // -Z
        default: { dir = vec3<f32>(0.0); }
    }
    return normalize(dir);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y || id.z >= 6u) {
        return;
    }

    let dir = get_direction_from_cubemap(vec2<f32>(id.xy), id.z, vec2<f32>(dims));

    let phi = atan2(dir.z, dir.x);
    let theta = asin(dir.y);
    let uv = vec2<f32>(phi * INV_TWO_PI + 0.5, 0.5 - theta * INV_PI);

    let input_dims = vec2<f32>(textureDimensions(input_tex));

    let sample_coord = vec2<u32>(clamp(uv * input_dims, vec2<f32>(0.0), input_dims - 1.0));
    let color = textureLoad(input_tex, sample_coord, 0);

    textureStore(output_tex, id.xy, id.z, color);
}