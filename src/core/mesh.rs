use crate::core::vertex::Vertex;

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn to_vertices(&self) -> Vec<Vertex> {
        self.vertices
            .iter()
            .enumerate()
            .map(|(i, &position)| Vertex {
                position,
                normal: self.normals[i],
                uv: self.uvs[i],
            })
            .collect()
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}
