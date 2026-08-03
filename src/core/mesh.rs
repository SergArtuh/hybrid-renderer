use wgpu::util::DeviceExt;

use crate::core::vertex::Vertex;

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
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
                tangent: self.tangents[i],
                uv: self.uvs[i],
            })
            .collect()
    }
}

pub struct Mesh {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,
}

impl Mesh {
    const PROCEDURAL_QUAD_MESH: Mesh = Mesh {
        vertex_buffer: None,
        index_buffer: None,
        index_count: 6,
    };
    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            index_count,
        }
    }

    pub fn from_data(device: &wgpu::Device, mesh_data: &MeshData) -> Self {
        if mesh_data.vertices.is_empty() || mesh_data.indices.is_empty() {
            return Self::PROCEDURAL_QUAD_MESH;
        } else {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh_data.to_vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh_data.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            Self {
                vertex_buffer: Some(vertex_buffer),
                index_buffer: Some(index_buffer),
                index_count: mesh_data.indices.len() as u32,
            }
        }
    }
}
