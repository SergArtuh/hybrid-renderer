import os
import json
import argparse
import numpy as np
import trimesh
import OpenEXR
import Imath
from PIL import Image

try:
    from tqdm import tqdm
except ImportError:
    def tqdm(iterable, desc=""):
        print(f"⏳ {desc}...")
        return iterable

def generate_sdf_from_glb(glb_file, output_base, grid_res=64):
    print(f"⏳ Loading model from {glb_file}...")
    scene_or_mesh = trimesh.load(glb_file)
    
    if isinstance(scene_or_mesh, trimesh.Scene):
        if len(scene_or_mesh.geometry) == 0:
            raise ValueError("❌ Error: GLB scene is empty!")
        
        meshes = []
        for node_name in scene_or_mesh.graph.nodes_geometry:
            matrix, geom_name = scene_or_mesh.graph.get(node_name)
            if geom_name in scene_or_mesh.geometry:
                submesh = scene_or_mesh.geometry[geom_name].copy()
                submesh.apply_transform(matrix)
                meshes.append(submesh)
        
        if len(meshes) > 0:
            mesh = trimesh.util.concatenate(meshes)
        else:
            mesh = scene_or_mesh.dump(concatenate=True)
    else:
        mesh = scene_or_mesh

    min_b = mesh.bounds[0]
    max_b = mesh.bounds[1]
    size = max_b - min_b
    center = (min_b + max_b) * 0.5

    PADDING_FACTOR = 1.25
    max_dim = max(size[0], size[1], size[2]) * PADDING_FACTOR
    half_size = max_dim * 0.5

    box_min = center - np.array([half_size, half_size, half_size], dtype=np.float32)
    box_max = center + np.array([half_size, half_size, half_size], dtype=np.float32)
    step = max_dim / grid_res

    print(f"📦 Model dimensions: X={size[0]:.2f}, Y={size[1]:.2f}, Z={size[2]:.2f}")
    print(f"📦 Cubic bounding box with padding (max_dim): {max_dim:.2f}")

    x_coords = box_min[0] + (np.arange(grid_res) + 0.5) * step
    y_coords = box_min[1] + (np.arange(grid_res) + 0.5) * step
    z_coords = box_min[2] + (np.arange(grid_res) + 0.5) * step

    xx, yy, zz = np.meshgrid(x_coords, y_coords, z_coords, indexing='ij')

    print("⏳ Building ProximityQuery acceleration tree...")
    query = trimesh.proximity.ProximityQuery(mesh)
    print("✅ Acceleration tree built.")

    print("⏳ Computing Signed Distance Fields (SDF) layer by layer...")
    grid_3d = np.zeros((grid_res, grid_res, grid_res), dtype=np.float32)

    for z in tqdm(range(grid_res), desc="SDF Layers"):
        z_coord = box_min[2] + (z + 0.5) * step
        
        xx, yy = np.meshgrid(x_coords, y_coords, indexing='ij')
        zz = np.full_like(xx, z_coord)
        slice_points = np.column_stack((xx.ravel(), yy.ravel(), zz.ravel()))
        
        dists = query.signed_distance(slice_points)
        dists = -dists
        
        grid_3d[z, :, :] = dists.reshape((grid_res, grid_res))

    grid_3d = np.transpose(grid_3d, (0, 2, 1))

    base_path = os.path.splitext(output_base)[0]

    box_size_val = float(box_max[0] - box_min[0])
    sdf_scale = 1.0 / box_size_val

    inv_scale = 1.0 / box_size_val

    inv_world_matrix = [
        [inv_scale, 0.0, 0.0, 0.0],
        [0.0, inv_scale, 0.0, 0.0],
        [0.0, 0.0, inv_scale, 0.0],
        [-float(center[0]) * inv_scale, -float(center[1]) * inv_scale, -float(center[2]) * inv_scale, 1.0]
    ]

    world_matrix = [
        [box_size_val, 0.0, 0.0, 0.0],
        [0.0, box_size_val, 0.0, 0.0],
        [0.0, 0.0, box_size_val, 0.0],
        [float(center[0]), float(center[1]), float(center[2]), 1.0]
    ]

    json_path = base_path + ".json"
    meta_data = {
        "grid_resolution": grid_res,
        "box_min": box_min.tolist(),
        "box_max": box_max.tolist(),
        "box_size": box_size_val,
        "sdf_scale": sdf_scale,
        "sdf_inv_world_matrix": inv_world_matrix
    }
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(meta_data, f, indent=4)
    print(f"✅ JSON metadata saved: {json_path}")

    raw_path = base_path + ".raw"
    grid_3d.tofile(raw_path)
    print(f"✅ RAW 3D texture saved: {raw_path} ({grid_res}x{grid_res}x{grid_res}, float32)")

    exr_path = base_path + ".exr"
    tiles = 8
    atlas_size = tiles * grid_res
    atlas = np.zeros((atlas_size, atlas_size), dtype=np.float32)

    for z in range(grid_res):
        tx = z % tiles
        ty = z // tiles
        slice_2d = grid_3d[z, :, :]
        atlas[ty * grid_res : (ty + 1) * grid_res, tx * grid_res : (tx + 1) * grid_res] = slice_2d

    os.makedirs(os.path.dirname(os.path.abspath(exr_path)), exist_ok=True)
    
    header = OpenEXR.Header(atlas_size, atlas_size)
    pixel_type = Imath.PixelType(Imath.PixelType.FLOAT)
    header['channels'] = {
        'R': Imath.Channel(pixel_type),
        'G': Imath.Channel(pixel_type),
        'B': Imath.Channel(pixel_type)
    }

    atlas_bytes = atlas.tobytes()
    out_file = OpenEXR.OutputFile(exr_path, header)
    out_file.writePixels({
        'R': atlas_bytes,
        'G': atlas_bytes,
        'B': atlas_bytes
    })
    out_file.close()
    print(f"✅ EXR atlas saved: {exr_path}")

    p_min, p_max = atlas.min(), atlas.max()
    normalized_atlas = np.clip((atlas - p_min) / (p_max - p_min + 1e-5), 0.0, 1.0)
    atlas_img_data = (normalized_atlas * 255).astype(np.uint8)
    
    png_path = base_path + "_preview.png"
    img = Image.fromarray(atlas_img_data)
    img.save(png_path)
    print(f"🖼️ PNG preview saved: {png_path}")

    inv_m = inv_world_matrix
    wm = world_matrix

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Standalone SDF texture generator from 3D model (.glb)")
    parser.add_argument("input", type=str, help="Path to input .glb file")
    parser.add_argument("-o", "--output", type=str, default="car_sdf.exr", help="Base path for output files")
    parser.add_argument("-r", "--resolution", type=int, default=64, help="Grid resolution (default 64)")

    args = parser.parse_args()
    generate_sdf_from_glb(args.input, args.output, args.resolution)