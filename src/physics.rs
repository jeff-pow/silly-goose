use crate::BORDER_RADIUS;
use std::f32::consts::PI;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Scene {
    pub static_vertices: Vec<Vertex>,
    pub static_indices: Vec<u32>,
    pub static_shape_offsets: Vec<u32>,

    pub dynamic_vertices: Vec<Vertex>,
    pub dynamic_indices: Vec<u32>,
    pub dynamic_shape_offsets: Vec<u32>,
    dynamic_circle_dims: Vec<(f32, [f32; 3])>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            static_vertices: Vec::new(),
            static_indices: Vec::new(),
            static_shape_offsets: Vec::new(),
            dynamic_vertices: Vec::new(),
            dynamic_indices: Vec::new(),
            dynamic_shape_offsets: Vec::new(),
            dynamic_circle_dims: Vec::new(),
        }
    }

    pub fn add_dynamic_shape(&mut self, shape: Shape, radius: f32, center: [f32; 3]) {
        self.dynamic_shape_offsets.push(self.dynamic_indices.len() as u32);

        let vertex_offset = self.dynamic_vertices.len();
        self.dynamic_vertices.extend(shape.vertices);

        self.dynamic_indices
            .extend(shape.indices.iter().map(|&v| v + vertex_offset as u32));
        self.dynamic_circle_dims.push((radius, center));
    }

    pub fn add_static_shape(&mut self, shape: Shape) {
        self.static_shape_offsets.push(self.static_indices.len() as u32);

        let vertex_offset = self.static_vertices.len();
        self.static_vertices.extend(shape.vertices);

        self.static_indices
            .extend(shape.indices.iter().map(|&v| v + vertex_offset as u32));
    }

    pub fn create_gpu_buffers(
        &self,
        device: &wgpu::Device,
    ) -> ((wgpu::Buffer, wgpu::Buffer), (wgpu::Buffer, wgpu::Buffer)) {
        let static_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.static_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let static_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.static_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let dynamic_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.dynamic_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let dynamic_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.dynamic_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        (
            (static_vertex_buffer, static_index_buffer),
            (dynamic_vertex_buffer, dynamic_index_buffer),
        )
    }

    pub fn create_3d_border(&mut self, radius: f32, num_subdivisions: u32, center: [f32; 3]) {
        let lat_steps = num_subdivisions;
        let lon_steps = num_subdivisions * 2;

        for lat in 0..=lat_steps {
            let theta = (lat as f32 * PI) / lat_steps as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=lon_steps {
                let phi = (lon as f32 * 2. * PI) / lon_steps as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = radius * sin_theta * cos_phi + center[0];
                let y = radius * sin_theta * sin_phi + center[1];
                let z = radius * cos_theta + center[2];

                let dot = Shape::sphere(0.015, 16, [x, y, z], [0.411, 0.411, 0.411, 0.3]);
                self.add_static_shape(dot);
            }
        }
    }

    pub fn create_2d_border(&mut self, radius: f32, num_subdivisions: u32, center: [f32; 3]) {
        let angle_increment = (2. * PI) / num_subdivisions as f32;
        let mut indices = Vec::new();

        for i in 0..num_subdivisions {
            let angle = i as f32 * angle_increment;
            let x = angle.cos() * radius + center[0];
            let y = angle.sin() * radius + center[1];
            let dot = Shape::sphere(0.015, 32, [x, y, 0.], [0.411, 0.411, 0.411, 0.3]);
            self.add_static_shape(dot);
        }

        for i in 0..num_subdivisions {
            let i0 = i;
            let i1 = (i + 1) % num_subdivisions;
            indices.extend([i0, i1, 0]);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Shape {
    pub fn sphere(radius: f32, num_subdivisions: u32, center: [f32; 3], color: [f32; 4]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let lat_steps = num_subdivisions;
        let lon_steps = num_subdivisions * 2;

        for lat in 0..=lat_steps {
            let theta = (lat as f32 * PI) / lat_steps as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=lon_steps {
                let phi = (lon as f32 * 2. * PI) / lon_steps as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = radius * sin_theta * cos_phi + center[0];
                let y = radius * sin_theta * sin_phi + center[1];
                let z = radius * cos_theta + center[2];

                let mut normal = [x - center[0], y - center[1], z - center[2]];
                let div = normal.iter().map(|&x| x * x).sum::<f32>().sqrt();
                normal.iter_mut().for_each(|x| *x /= div);

                vertices.push(Vertex {
                    position: [x, y, z],
                    color,
                    normal,
                });
            }
        }

        for lat in 0..lat_steps {
            for lon in 0..lon_steps {
                let current = lat * (lon_steps + 1) + lon;
                let next = current + lon_steps + 1;

                indices.extend(&[current, next, current + 1]);
                indices.extend(&[next, next + 1, current + 1]);
            }
        }

        Shape { vertices, indices }
    }

    pub fn polygon(radius: f32, num_subdivisions: u32, center: [f32; 3], color: [f32; 4]) -> Self {
        let mut vertices = Vec::new();
        let angle_increment = (2. * PI) / num_subdivisions as f32;
        let mut indices = Vec::new();

        for i in 0..num_subdivisions {
            let angle = i as f32 * angle_increment;
            let x = angle.cos() * radius + center[0];
            let y = angle.sin() * radius + center[1];
            vertices.push(Vertex {
                position: [x, y, 0.],
                color,
                normal: [0., 1., 0.],
            });
        }

        for i in 0..num_subdivisions {
            let i0 = i;
            let i1 = (i + 1) % num_subdivisions;
            indices.extend([i0, i1, 0]);
        }

        Shape { vertices, indices }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
