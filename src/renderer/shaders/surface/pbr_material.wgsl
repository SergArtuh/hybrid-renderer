struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)  normal: vec3<f32>,
    @location(1)  uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
};

struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
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


@vertex
fn vs_main(
    //@builtin(vertex_index) in_vertex_index: u32
    vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    //out.clip_position = camera.proj_view * (model.model_matrix * vec4<f32>(vertex.position, 1.0) +  vec4<f32>(0.0, 0.6, 0.0, 0.0));
    //out.clip_position = camera.proj_view * model.model_matrix * vec4<f32>(vertex.position, 1.0);

    let world_pos = model.model_matrix * vec4<f32>(vertex.position, 1.0);
    
    out.world_position = world_pos.xyz;

    out.clip_position = camera.proj_view * world_pos;

    
    out.normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Сэмплим текстуры модели
    let base_color = textureSample(t_base_color, common_sampler, in.uv).rgb;
    let orm = textureSample(t_metallic_roughness, common_sampler, in.uv);
    
    // В glTF / стандартном ORM: 
    // R = Occlusion (не всегда используется так), G = Roughness, B = Metallic
    let roughness = orm.g;
    let metallic  = orm.b;

    // 2. Вектора (ВАЖНО: нормализуем reflect_dir после reflect!)
    let N = normalize(in.normal);
    let view_dir = normalize(in.world_position - camera.position.xyz);
    let reflect_dir = normalize(reflect(view_dir, N));
    
    // 3. Диффузное освещение (Irradiance) из кубмапы
    let hdr_color_irradiance = textureSample(t_irradiance, s_texture, N).rgb;
    
    // 4. Спекулярное освещение (Specular) на основе динамического LOD
    // Узнаем максимальный мип-уровень у сгенерированной кубмапы спекуляра
    let max_mip = f32(textureNumLevels(t_specular) - 1u);
    
    // Переводим roughness материала в уровень мипа (используем квадратичный шаг, как в генераторе)
    let lod = (roughness * roughness) * max_mip;
    let hdr_color_specular = textureSampleLevel(t_specular, s_texture, reflect_dir, lod).rgb;
    
    // 5. Правильное PBR-смешивание (Псевдо-Блики)
    // Диэлектрики (пластик, лак, резина) отражают мало (около 4%), остальное — их базовый цвет.
    // Металлы не имеют диффузного цвета, они окрашивают само отражение в свой base_color.
    
    // Диффузная составляющая (зависит от металличности)
    let diffuse = hdr_color_irradiance * base_color * (1.0 - metallic);
    
    // Спекулярная составляющая (металлы подкрашивают отражение своим цветом)
    let specular_tint = mix(vec3<f32>(0.04), base_color, metallic);
    let specular = hdr_color_specular * specular_tint;
    
    // Итоговый HDR цвет сцены
    let hdr_color = diffuse + specular;
    
    // 6. Тономаппинг (Reinhard) и вывод
    let color = hdr_color / (hdr_color + vec3<f32>(1.0));
    
    return vec4<f32>(color, 1.0);
}

/* 
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color = textureSample(t_base_color, common_sampler, in.uv);
    var normal = textureSample(t_normal, common_sampler, in.uv);
    var metallic_roughness = textureSample(t_metallic_roughness, common_sampler, in.uv);
    var occlusion = textureSample(t_occlusion, common_sampler, in.uv);
    var emissive = textureSample(t_emissive, common_sampler, in.uv);

    let orm = textureSample(t_metallic_roughness, common_sampler, in.uv);
    let roughness = orm.g;
    let metallic = orm.b;

    let view_dir = normalize(in.world_position - camera.position.xyz);
    let reflect_dir = reflect(view_dir, normalize(in.normal));
    
    //let hdr_color_reflect = textureSample(env_cubemap, s_texture, reflect_dir);
    let hdr_color_irradiance = textureSample(t_irradiance, s_texture, normalize(in.normal));
    let hdr_color_specular = textureSampleLevel(t_specular, s_texture, normalize(reflect_dir), 4.0);
    let hdr_color = hdr_color_specular * 0.5 + hdr_color_irradiance * 0.5;

    
    let color = hdr_color.rgb / (hdr_color.rgb + vec3(1.0));
    return vec4<f32>(color, 1.0);
    //return vec4<f32>(in.clip_position.xyz / in.clip_position.w, 1.0);
    //return vec4<f32>(roughness, roughness, roughness, 1.0);
    //return vec4<f32>(metallic, metallic, metallic, 1.0);
    //return vec4<f32>(occlusion.rgb, 1.0);
    //return vec4<f32>(emissive.rgb, 1.0);
}
*/