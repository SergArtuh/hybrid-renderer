struct CameraUniform {
    proj_view: mat4x4<f32>,
    inv_skybox_view_proj: mat4x4<f32>,
    position: vec4<f32>, 
}

struct SDFTransform {
    inv_world: mat4x4<f32>,
    tiles_per_side: u32
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var env_cubemap: texture_cube<f32>;
@group(0) @binding(2) var s_texture: sampler;
@group(0) @binding(3) var t_irradiance: texture_cube<f32>;
@group(0) @binding(4) var t_specular: texture_cube<f32>;

@group(1) @binding(0) var input_normal_tex: texture_2d<f32>;
@group(1) @binding(1) var input_depth_tex: texture_depth_2d;
@group(1) @binding(2) var output_ao_tex: texture_storage_2d<rgba16float, write>;
@group(1) @binding(3) var nearest_sampler: sampler;
@group(1) @binding(4) var sdf_tex: texture_2d<f32>;
@group(1) @binding(5) var<uniform> sdf_transform: SDFTransform;

fn ray_unit_box_intersect(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec2<f32> {
    let box_min = vec3<f32>(-0.5);
    let box_max = vec3<f32>(0.5);
    
    let inv_dir = 1.0 / ray_dir;
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;
    
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    
    let t_near = max(max(tmin.x, tmin.y), tmin.z);
    let t_far = min(min(tmax.x, tmax.y), tmax.z);
    
    return vec2<f32>(t_near, t_far);
}

fn sample_sdf_atlas(uvw: vec3<f32>) -> f32 {
    let grid_res = 64.0;
    let tiles_per_side = 8.0;

    let z_coord = clamp(uvw.z, 0.0, 1.0) * (grid_res - 1.0);
    let z0 = u32(floor(z_coord));
    let z1 = min(z0 + 1u, u32(grid_res - 1.0));
    let z_fract = fract(z_coord);

    let val0 = sample_tile_bilinear(uvw.xy, z0);
    let val1 = sample_tile_bilinear(uvw.xy, z1);

    return mix(val0, val1, z_fract);
}

fn sample_tile_bilinear(xy: vec2<f32>, z_slice: u32) -> f32 {
    let grid_res = 64.0;
    let tiles_per_side = 8.0;

    let tile_x = z_slice % u32(tiles_per_side);
    let tile_y = z_slice / u32(tiles_per_side);

    let pixel_pos = xy * grid_res - 0.5;
    let base_px = floor(pixel_pos);
    let f = fract(pixel_pos);

    let base_x = f32(tile_x) * grid_res;
    let base_y = f32(tile_y) * grid_res;

    let max_idx = grid_res - 1.0;
    
    let x0 = clamp(base_px.x, 0.0, max_idx);
    let x1 = clamp(base_px.x + 1.0, 0.0, max_idx);
    let y0 = clamp(base_px.y, 0.0, max_idx);
    let y1 = clamp(base_px.y + 1.0, 0.0, max_idx);

    let p00 = textureLoad(sdf_tex, vec2<u32>(u32(base_x + x0), u32(base_y + y0)), 0).r;
    let p10 = textureLoad(sdf_tex, vec2<u32>(u32(base_x + x1), u32(base_y + y0)), 0).r;
    let p01 = textureLoad(sdf_tex, vec2<u32>(u32(base_x + x0), u32(base_y + y1)), 0).r;
    let p11 = textureLoad(sdf_tex, vec2<u32>(u32(base_x + x1), u32(base_y + y1)), 0).r;

    let ix0 = mix(p00, p10, f.x);
    let ix1 = mix(p01, p11, f.x);
    return mix(ix0, ix1, f.y);
}

fn get_sdf_value(current_pos: vec3<f32>) -> f32 {
    let local_uvw = current_pos + vec3<f32>(0.5);

    if (!all(local_uvw >= vec3<f32>(0.0)) || !all(local_uvw <= vec3<f32>(1.0))) {
        return 1.0; 
    }
    
    return sample_sdf_atlas(local_uvw);
}

fn calc_sdf_normal(pos: vec3<f32>) -> vec3<f32> {
    let h = 0.001;
    let k = vec3<f32>(1.0, -1.0, -1.0);
    
    return normalize(
        k.xyy * get_sdf_value(pos + k.xyy * h) +
        k.yyx * get_sdf_value(pos + k.yyx * h) +
        k.yxy * get_sdf_value(pos + k.yxy * h) +
        k.xxx * get_sdf_value(pos + k.xxx * h)
    );
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_ao_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) / vec2<f32>(dims);
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0);
    let target_world = camera.inv_skybox_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    
    let world_ray_origin = camera.position.xyz;
    let world_ray_dir = normalize(target_world.xyz / target_world.w - world_ray_origin);
    
    let local_ray_origin = (sdf_transform.inv_world * vec4<f32>(world_ray_origin, 1.0)).xyz;
    let local_ray_dir = (sdf_transform.inv_world * vec4<f32>(world_ray_dir, 0.0)).xyz;

    let hit = ray_unit_box_intersect(local_ray_origin, local_ray_dir);

    var color = vec4<f32>(0.05, 0.05, 0.05, 1.0);

    if (hit.x <= hit.y && hit.y > 0.0) {
        let t_start = max(hit.x, 0.0);
        let t_end = hit.y;

        /*
        // MIN DISTANCE MODE START
        let num_steps = 96;
        let step_size = (t_end - t_start) / f32(num_steps);
        
        var min_dist = 10.0;
        var max_depth = 0.0;
        var hit_inside = false; 
        
        var t = t_start;
        for (var i = 0; i < num_steps; i++) {
            let current_pos = local_ray_origin + local_ray_dir * t;
            let dist = get_sdf_value(current_pos);
            
            if (dist > 0.0 && dist < min_dist) {
                min_dist = dist;
            }
            if (dist < 0.0) {
                hit_inside = true;
                if (dist < max_depth) {
                    max_depth = dist;
                }
            }
            
            t += step_size;
            if (t > t_end) {
                break;
            }
        }

        let reach = 0.3; 
        var surf_color = vec3<f32>(1.0);
        
        let outline_dist = 0.03; // set 0 to disable outline
        if (hit_inside) {
            surf_color = vec3<f32>(0.0, 0.0, 1.0);
        } else {
             if (min_dist < outline_dist) {
                surf_color = vec3<f32>(1.0, 1.0, 1.0);
            } else {
                let t_val = clamp(min_dist / reach, 0.0, 1.0);
                surf_color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), t_val); 
            }
        }
        
        color = vec4<f32>(surf_color, 1.0);
        // MIN DISTANCE MODE END
        */

        // NORMAL VISUALISE START
        let max_steps = 64;
        var t = t_start;
        var hit_surface = false;
        var final_pos = vec3<f32>(0.0);

        for (var i = 0; i < max_steps; i++) {
            let current_pos = local_ray_origin + local_ray_dir * t;
            let dist = get_sdf_value(current_pos);
            
            if (dist <= 0.001) {
                hit_surface = true;
                final_pos = current_pos;
                break;
            }

            t += dist;

            if (t > t_end) {
                break;
            }
        }

        if (hit_surface) {
            let normal = calc_sdf_normal(final_pos);
            let normal_color = normal * 0.5 + 0.5;
            color = vec4<f32>(normal_color, 1.0);
        }
        // NORMAL VISUALISE END
    }

    textureStore(output_ao_tex, id.xy, color);
}