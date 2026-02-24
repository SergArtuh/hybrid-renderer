use std::f32::consts::PI;

use crate::core::mesh::MeshData;

pub struct MeshUtil;

impl MeshUtil {
    pub fn new_sphere(radius: f32, latitude_count: u32, longitude_count: u32) -> MeshData {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=latitude_count {
            let theta = i as f32 * PI / latitude_count as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for j in 0..=longitude_count {
                let phi = j as f32 * 2.0 * PI / longitude_count as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                normals.push([x, y, z]);

                vertices.push([x * radius, y * radius, z * radius]);

                let u = j as f32 / longitude_count as f32;
                let v = i as f32 / latitude_count as f32;
                uvs.push([u, v]);
            }
        }

        for i in 0..latitude_count {
            for j in 0..longitude_count {
                let first = i * (longitude_count + 1) + j;
                let second = first + longitude_count + 1;
                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        MeshData {
            vertices,
            normals,
            uvs,
            indices,
        }
    }

    pub fn new_cube(size: f32) -> MeshData {
        let mut vertices = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        let half = size / 2.0;

        let faces = [
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    [-half, -half, half], // BL
                    [half, -half, half],  // BR
                    [half, half, half],   // TR
                    [-half, half, half],  // TL
                ],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    [half, -half, -half],
                    [-half, -half, -half],
                    [-half, half, -half],
                    [half, half, -half],
                ],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [
                    [-half, half, half],
                    [half, half, half],
                    [half, half, -half],
                    [-half, half, -half],
                ],
            ),
            // Bottom (-Y)
            (
                [0.0, -1.0, 0.0],
                [
                    [-half, -half, -half],
                    [half, -half, -half],
                    [half, -half, half],
                    [-half, -half, half],
                ],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    [half, -half, half],
                    [half, -half, -half],
                    [half, half, -half],
                    [half, half, half],
                ],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    [-half, -half, -half],
                    [-half, -half, half],
                    [-half, half, half],
                    [-half, half, -half],
                ],
            ),
        ];

        let mut index_offset = 0;

        for (normal, face_verts) in faces.iter() {
            for (i, &pos) in face_verts.iter().enumerate() {
                vertices.push(pos);
                normals.push(*normal);

                match i {
                    0 => uvs.push([0.0, 1.0]),
                    1 => uvs.push([1.0, 1.0]),
                    2 => uvs.push([1.0, 0.0]),
                    3 => uvs.push([0.0, 0.0]),
                    _ => unreachable!(),
                }
            }

            indices.push(index_offset + 0);
            indices.push(index_offset + 1);
            indices.push(index_offset + 2);

            indices.push(index_offset + 2);
            indices.push(index_offset + 3);
            indices.push(index_offset + 0);

            index_offset += 4;
        }

        MeshData {
            vertices,
            normals,
            uvs,
            indices,
        }
    }
}
