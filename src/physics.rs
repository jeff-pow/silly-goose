use std::f32::consts::PI;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Shapes {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,       // or u16, depending on the number of vertices
    pub shape_offsets: Vec<u32>, // Offset into vertex and index buffers for each shape
}

impl Shapes {
    pub fn new() -> Self {
        Shapes {
            vertices: Vec::new(),
            indices: Vec::new(),
            shape_offsets: Vec::new(),
        }
    }

    pub fn add_shape(&mut self, shape_vertices: Vec<Vertex>, shape_indices: Vec<u32>) {
        let shape_offset = self.vertices.len();
        // Store the offset into the **index** buffer where this shape's indices begin.
        self.shape_offsets.push(self.indices.len() as u32);

        //Extend the global vertex list
        self.vertices.extend(shape_vertices);

        //Extend the index buffer, the vertices are already adjusted
        self.indices
            .extend(shape_indices.iter().map(|&v| v + shape_offset as u32));
    }

    pub fn create_gpu_buffers(&self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn update_vertex_colors(&mut self) {
        self.vertices.iter_mut().for_each(Vertex::update);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn update(&mut self) {
        for color in &mut self.color {
            *color = 1f32.min(*color + 0.01);
        }
    }
}

pub fn polygon(radius: f32, num_subdivisions: u32, center: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let angle_increment = (2. * PI) / num_subdivisions as f32;
    let mut indices = Vec::new();

    for i in 0..num_subdivisions {
        let angle = i as f32 * angle_increment;
        let x = angle.cos() * radius + center[0];
        let y = angle.sin() * radius + center[1];
        vertices.push(Vertex {
            position: [x, y, 0.],
            color: [1., 0., 0.],
        });
    }

    for i in 0..num_subdivisions {
        let i0 = i;
        let i1 = (i + 1) % num_subdivisions;
        indices.extend([i0, i1, 0]);
    }

    (vertices, indices)
}
