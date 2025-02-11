#[repr(C)]
#[derive(Clone, Debug)]
pub struct SimulationData {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,       // or u16, depending on the number of vertices
    shape_offsets: Vec<u32>, // Offset into vertex and index buffers for each shape
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

pub fn generate_circle_vertices(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity((segments + 1) as usize);
    let mut indices = Vec::with_capacity((segments * 3) as usize);

    // Center vertex
    vertices.push(Vertex {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 1.0, 1.0],
    });

    // Generate vertices around the circle
    for i in 0..segments {
        let angle = (i as f32 * 2.0 * std::f32::consts::PI) / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();

        vertices.push(Vertex {
            position: [x, y, 0.0],
            color: [1.0, 1.0, 1.0], // White color
        });

        // Generate indices for triangles
        if i < segments - 1 {
            indices.extend_from_slice(&[
                0,              // Center
                (i + 1) as u16, // Current vertex
                (i + 2) as u16, // Next vertex
            ]);
        } else {
            // Connect last vertex back to first
            indices.extend_from_slice(&[
                0,               // Center
                segments as u16, // Last vertex
                1,               // Back to first vertex
            ]);
        }
    }

    (vertices, indices)
}
