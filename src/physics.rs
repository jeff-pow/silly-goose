use crate::{BORDER_CENTER, BORDER_RADIUS};
use glam::{Vec3, Vec4};
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct PhysicsBody {
    pub pos: Vec3,
    pub radius: f32,
    pub velocity: Vec3,
    pub mass: f32,
}

impl PhysicsBody {
    pub fn new(pos: Vec3, radius: f32) -> Self {
        Self {
            pos,
            radius,
            velocity: Vec3::ZERO,
            mass: 1.0,
        }
    }

    pub fn keep_within_border(&mut self) {
        let distance_from_center = self.pos.distance(BORDER_CENTER);
        if distance_from_center + self.radius > BORDER_RADIUS {
            let dir = (self.pos - BORDER_CENTER).normalize();
            self.pos = BORDER_CENTER + dir * (BORDER_RADIUS - self.radius);

            let normal = -dir;
            let vel_along_normal = self.velocity.dot(normal);
            self.velocity -= 2.0 * vel_along_normal * normal;
            self.velocity *= 0.95; // Elasticity
        }
    }

    pub fn collide_with(&mut self, other: &mut PhysicsBody) {
        let distance = self.pos.distance(other.pos);

        if distance < self.radius + other.radius {
            let normal = (other.pos - self.pos).normalize();

            let relative_velocity = other.velocity - self.velocity;

            let velocity_along_normal = relative_velocity.dot(normal);

            if velocity_along_normal > 0.0 {
                return;
            }

            let restitution = 0.95; // 95% elastic collision
            let mut impulse_scalar = -(1.0 + restitution) * velocity_along_normal;
            impulse_scalar /= (1.0 / self.mass) + (1.0 / other.mass);

            let impulse = impulse_scalar * normal;
            self.velocity -= impulse / self.mass;
            other.velocity += impulse / other.mass;

            let overlap = (self.radius + other.radius) - distance;
            let separation_vector = normal * (overlap * 0.5);
            self.pos -= separation_vector;
            other.pos += separation_vector;
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Scene {
    pub physics_bodies: Vec<PhysicsBody>,

    pub static_meshes: Vec<Mesh>,
    pub dynamic_meshes: Vec<Mesh>,

    // Separate counters for static/dynamic
    next_static_vertex: usize,
    next_static_index: usize,
    next_dynamic_vertex: usize,
    next_dynamic_index: usize,
}

impl Scene {
    pub fn create_3d_border(&mut self, radius: f32, subdivisions: u32, center: Vec3) {
        let lat_steps = subdivisions;
        let lon_steps = subdivisions * 2;

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

                let mut dot = Mesh::sphere(0.015, 16, Vec3::new(x, y, z), Vec4::new(0.411, 0.411, 0.411, 0.3));
                let vertex_offset = self.next_static_vertex;
                dot.indices.iter_mut().for_each(|i| *i += vertex_offset as u32);
                dot.buffer_offset = self.next_static_index;

                // Update static offsets
                self.next_static_vertex += dot.vertices.len();
                self.next_static_index += dot.indices.len();

                self.static_meshes.push(dot);
            }
        }
    }

    pub fn add_ball(&mut self, radius: f32, center: Vec3, color: Vec4) {
        let mut mesh = Mesh::sphere(radius, 8, center, color);

        let vertex_offset = self.next_dynamic_vertex;
        mesh.indices.iter_mut().for_each(|i| *i += vertex_offset as u32);
        mesh.buffer_offset = self.next_dynamic_index;

        self.next_dynamic_vertex += mesh.vertices.len();
        self.next_dynamic_index += mesh.indices.len();

        self.dynamic_meshes.push(mesh);

        self.physics_bodies.push(PhysicsBody::new(center, radius));
    }

    pub fn update_physics(&mut self, dt: f32) {
        self.physics_bodies.iter_mut().for_each(|b| {
            let force = Vec3::new(0.0, -9.8 * b.mass, 0.0);
            b.velocity += force * dt / b.mass;
        });

        self.physics_bodies.iter_mut().for_each(|b| b.pos += b.velocity * dt);

        const SOLVER_ITERATIONS: usize = 3;
        for _ in 0..SOLVER_ITERATIONS {
            self.physics_bodies.iter_mut().for_each(PhysicsBody::keep_within_border);

            for i in 0..self.physics_bodies.len() {
                let (first, rest) = self.physics_bodies.split_at_mut(i);
                for b1 in first {
                    for b2 in rest.iter_mut() {
                        b1.collide_with(b2);
                    }
                }
            }
        }
    }

    pub fn update_dynamic_vertices(&mut self) {
        for (mesh, body) in self.dynamic_meshes.iter_mut().zip(&self.physics_bodies) {
            let offset = body.pos - mesh.center;
            for vertex in &mut mesh.vertices {
                vertex.position = [
                    vertex.position[0] + offset[0],
                    vertex.position[1] + offset[1],
                    vertex.position[2] + offset[2],
                ];
            }
            mesh.center = body.pos;
        }
    }

    pub fn static_vertices(&self) -> Vec<Vertex> {
        self.static_meshes.iter().flat_map(|m| m.vertices.clone()).collect()
    }

    pub fn static_indices(&self) -> Vec<u32> {
        self.static_meshes.iter().flat_map(|m| m.indices.clone()).collect()
    }

    pub fn dynamic_vertices(&self) -> Vec<Vertex> {
        self.dynamic_meshes.iter().flat_map(|m| m.vertices.clone()).collect()
    }

    pub fn dynamic_indices(&self) -> Vec<u32> {
        self.dynamic_meshes.iter().flat_map(|m| m.indices.clone()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub buffer_offset: usize,
    center: Vec3,
}

impl Mesh {
    pub fn sphere(radius: f32, num_subdivisions: u32, center: Vec3, color: Vec4) -> Self {
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

                vertices.push(Vertex::new(
                    Vec3::from_array([x, y, z]),
                    color,
                    Vec3::from_array(normal),
                ));
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

        Self {
            vertices,
            indices,
            buffer_offset: 0,
            center,
        }
    }

    #[expect(unused)]
    pub fn polygon(radius: f32, num_subdivisions: u32, center: Vec3, color: Vec4, buffer_offset: usize) -> Self {
        let mut vertices = Vec::new();
        let angle_increment = (2. * PI) / num_subdivisions as f32;
        let mut indices = Vec::new();

        for i in 0..num_subdivisions {
            let angle = i as f32 * angle_increment;
            let x = angle.cos() * radius + center[0];
            let y = angle.sin() * radius + center[1];
            vertices.push(Vertex::new(Vec3::new(x, y, 0.), color, Vec3::new(0., 1., 0.)));
        }

        for i in 0..num_subdivisions {
            let i0 = i;
            let i1 = (i + 1) % num_subdivisions;
            indices.extend([i0, i1, 0]);
        }

        Self {
            vertices,
            indices,
            buffer_offset,
            center,
        }
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

    pub fn new(position: Vec3, color: Vec4, normal: Vec3) -> Self {
        Self {
            position: position.to_array(),
            color: color.to_array(),
            normal: normal.to_array(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
