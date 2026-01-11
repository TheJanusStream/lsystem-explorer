use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub struct LSystemMeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<u32>,
    resolution: u32,
}

impl Default for LSystemMeshBuilder {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            resolution: 8,
        }
    }
}

impl LSystemMeshBuilder {
    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }

    /// Generates a ring of vertices at the given transform with the given radius.
    /// Returns the index of the first vertex in this ring.
    pub fn add_ring(&mut self, transform: Transform, radius: f32) -> u32 {
        let start_index = self.positions.len() as u32;
        let rotation = transform.rotation;
        let translation = transform.translation;

        for i in 0..=self.resolution {
            let theta = (i as f32 / self.resolution as f32) * std::f32::consts::TAU;
            let (sin, cos) = theta.sin_cos();

            // Ring in local XZ plane (Turtle moves up Y)
            let local_pos = Vec3::new(cos * radius, 0.0, sin * radius);
            let world_pos = translation + (rotation * local_pos);
            let normal = rotation * Vec3::new(cos, 0.0, sin);

            self.positions.push(world_pos);
            self.normals.push(normal);
        }

        start_index
    }

    /// Connects two existing rings with triangles.
    pub fn connect_rings(&mut self, bottom_ring_start: u32, top_ring_start: u32) {
        let segs = self.resolution;

        for i in 0..segs {
            let bottom_curr = bottom_ring_start + i;
            let bottom_next = bottom_ring_start + i + 1;
            let top_curr = top_ring_start + i;
            let top_next = top_ring_start + i + 1;

            self.indices.push(bottom_curr);
            self.indices.push(top_curr);
            self.indices.push(bottom_next);

            self.indices.push(bottom_next);
            self.indices.push(top_curr);
            self.indices.push(top_next);
        }
    }
}
