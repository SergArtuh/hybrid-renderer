struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) view_dir: vec3<f32>,
};

struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

struct SkydomeUniform {
    data: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var t_cubemap: texture_cube<f32>;
@group(0) @binding(2) var s_texture: sampler;
@group(0) @binding(3) var t_irradiance: texture_cube<f32>;
@group(0) @binding(4) var t_specular: texture_cube<f32>;

@group(2) @binding(2) var<uniform> skydome_uniform: SkydomeUniform;


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2(-1.0,  1.0), vec2( 1.0,  1.0), vec2( 1.0, -1.0),
        vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2(-1.0, -1.0)
    );

    var out: VertexOutput;
    let p = pos[in_vertex_index];
    out.clip_position = vec4<f32>(p, 1.0, 1.0);
    let unprojected = camera.inv_skybox_view_proj * vec4<f32>(p, 1.0, 1.0);
    out.view_dir = unprojected.xyz / unprojected.w;
    return out;
}

fn get_ray_sphere_t(ray_origin: vec3<f32>, ray_dir: vec3<f32>, sphere_center: vec3<f32>, sphere_radius: f32) -> f32 {
    let oc = ray_origin - sphere_center;
    let b = dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere_radius * sphere_radius;
    let d = b * b - c;
    let s = sqrt(max(d, 0.0));
    let t = -b + s;
    return select(-1.0, t, d >= 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(in.view_dir);
    let cam_pos = camera.position.xyz;

    let radius = skydome_uniform.data.x;
    let dome_factor = skydome_uniform.data.y;
    
    let sphere_offset_y = radius * (1.0 - dome_factor);
    let sphere_center = vec3<f32>(0.0, sphere_offset_y, 0.0);

    let t_sphere = get_ray_sphere_t(cam_pos, view_dir, sphere_center, radius);
    let t_floor = -cam_pos.y / (view_dir.y - 0.000001);
    let is_floor = select(0.0, 1.0, t_floor > 0.0 && t_floor < t_sphere);
    let t = mix(t_sphere, t_floor, is_floor);
    let hit_point = cam_pos + view_dir * t;
    let final_dir = normalize(hit_point - sphere_center);

    let hdr_color = textureSample(t_cubemap, s_texture, final_dir);
    //let hdr_color = textureSampleLevel(t_cubemap, s_texture, final_dir, 3.0);
    //let hdr_color = textureSampleLevel(t_specular, s_texture, final_dir, 3.0);
    let tone_mapped_rgb = hdr_color.rgb / (hdr_color.rgb + vec3(1.0));
    
    return vec4<f32>(tone_mapped_rgb, 1.0);
}