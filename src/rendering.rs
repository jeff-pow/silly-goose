use crate::physics::Mesh;
use crate::Scene;
use wgpu::util::DeviceExt;

pub struct BufferManager {
    pub static_vertex_buffer: wgpu::Buffer,
    pub static_index_buffer: wgpu::Buffer,
    pub dynamic_vertex_buffer: wgpu::Buffer,
    pub dynamic_index_buffer: wgpu::Buffer,
}

impl BufferManager {
    pub fn new(device: &wgpu::Device, scene: &Scene) -> Self {
        let static_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&scene.static_vertices()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let static_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&scene.static_indices()),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let dynamic_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&scene.dynamic_vertices()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let dynamic_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&scene.dynamic_indices()),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            static_vertex_buffer,
            static_index_buffer,
            dynamic_vertex_buffer,
            dynamic_index_buffer,
        }
    }

    pub fn update_dynamic_buffers(&mut self, queue: &wgpu::Queue, scene: &Scene) {
        let all_vertices = scene
            .dynamic_meshes
            .iter()
            .flat_map(|m| &m.vertices)
            .copied()
            .collect::<Vec<_>>();

        let all_indices: Vec<u32> = scene.dynamic_meshes.iter().flat_map(|m| &m.indices).copied().collect();

        queue.write_buffer(&self.dynamic_vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        queue.write_buffer(&self.dynamic_index_buffer, 0, bytemuck::cast_slice(&all_indices));
    }
}

pub fn render_objects(
    render_pass: &mut wgpu::RenderPass,
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    meshes: &[Mesh],
) {
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    for mesh in meshes {
        render_pass.draw_indexed(
            mesh.buffer_offset as u32..(mesh.buffer_offset + mesh.indices.len()) as u32,
            0,
            0..1,
        );
    }
}
