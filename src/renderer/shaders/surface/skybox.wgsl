struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) view_dir: vec3<f32>,
};

struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var t_cubemap: texture_cube<f32>;
@group(2) @binding(1) var s_texture: sampler;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos: vec2<f32>;
    switch in_vertex_index {
        case 0u, 3u: { pos = vec2<f32>(-1.0,  1.0); }
        case 1u:     { pos = vec2<f32>( 1.0,  1.0); }
        case 2u, 4u: { pos = vec2<f32>( 1.0, -1.0); }
        case 5u:     { pos = vec2<f32>(-1.0, -1.0); }
        default:     { pos = vec2<f32>( 0.0,  0.0); }
    }

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 1.0, 1.0);
    let unprojected = camera.inv_skybox_view_proj * vec4<f32>(pos, 1.0, 1.0);
    out.view_dir = unprojected.xyz / unprojected.w;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(in.view_dir);
    let cam_pos = camera.position.xyz;
    let h = cam_pos.y;
    let floor_scale = 3.0;
    
    var final_dir = view_dir;
    if (view_dir.y < -0.001) {
        let dist = -h / view_dir.y;
        let world_intersection = cam_pos + view_dir * dist;

        if (world_intersection.x < floor_scale && world_intersection.x > -floor_scale && 
            world_intersection.z < floor_scale && world_intersection.z > -floor_scale) {
            final_dir = vec3<f32>(world_intersection.x / floor_scale, -1.0, world_intersection.z / floor_scale);
        }
    }


    let hdr_color = textureSample(t_cubemap, s_texture, final_dir);
    let tone_mapped_rgb = hdr_color.rgb / (hdr_color.rgb + vec3<f32>(1.0));
    return vec4<f32>(tone_mapped_rgb, 1.0);
    //return vec4<f32>(final_dir.x, final_dir.y, final_dir.z, 1.0);
}

/* 
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(in.view_dir);
    let cam_pos = camera.position.xyz;
    let h = cam_pos.y; 
    
    var final_dir = view_dir;

    if (view_dir.y < -0.001) {
        // 1. Находим точку на бесконечном полу
        let dist = -h / view_dir.y;
        let world_intersection = cam_pos + view_dir * dist;
        
        // 2. Вводим ограничение радиуса
        // Это "размер" твоего пола в метрах вокруг центра (0,0,0)
        let projected_radius = 500.0; 
        
        // Считаем расстояние от центра мира до точки пересечения
        let radial_dist = length(world_intersection.xz);
        
        // 3. Вычисляем направление для сэмплирования
        // Вместо бесконечного растяжения, мы ограничиваем вектор world_intersection
        // так, чтобы он не уходил дальше projected_radius
        let limited_p = normalize(world_intersection) * min(radial_dist, projected_radius);
        
        // Подмешиваем высоту купола, чтобы придать объем
        let dome_height = 30.0;
        let floor_dir = normalize(limited_p + vec3<f32>(0.0, dome_height, 0.0));
        
        // 4. Плавный переход
        // Чем ближе мы к горизонту (или чем дальше от центра), тем больше берем оригинальный view_dir
        let blend = smoothstep(projected_radius, projected_radius * 1.5, radial_dist);
        final_dir = mix(floor_dir, view_dir, blend);
    }

    let hdr_color = textureSample(t_cubemap, s_texture, final_dir);
    let tone_mapped_rgb = hdr_color.rgb / (hdr_color.rgb + vec3<f32>(1.0));
    return vec4<f32>(tone_mapped_rgb, 1.0);
}
*/