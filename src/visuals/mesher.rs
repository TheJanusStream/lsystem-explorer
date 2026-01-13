use crate::visuals::skeleton::Skeleton;
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
    pub fn build(mut self, skeleton: &Skeleton) -> Mesh {
        for strand in &skeleton.strands {
            if strand.len() < 2 {
                continue;
            }
            self.process_strand(strand);
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }

    fn process_strand(&mut self, points: &[crate::visuals::skeleton::SkeletonPoint]) {
        let points_count = points.len();
        let mut ring_start_indices = Vec::new();

        // 1. Initialize Frame using the first point's Turtle rotation.
        let last_tangent = (points[1].position - points[0].position).normalize_or_zero();

        // Initial orientation from the Turtle (Important for setting the "Up" relative to the ground)
        let mut current_rotation = points[0].rotation;

        // Ensure the initial rotation actually aligns with the first path segment
        let initial_turtle_forward = current_rotation * Vec3::Y;
        let initial_correction = Quat::from_rotation_arc(initial_turtle_forward, last_tangent);
        current_rotation = initial_correction * current_rotation;

        for i in 0..points_count {
            let curr = points[i];

            // Calculate Miter Tangent (Bisector)
            let miter_tangent = if i == 0 {
                (points[i + 1].position - curr.position).normalize_or_zero()
            } else if i == points_count - 1 {
                (curr.position - points[i - 1].position).normalize_or_zero()
            } else {
                let v_in = (curr.position - points[i - 1].position).normalize_or_zero();
                let v_out = (points[i + 1].position - curr.position).normalize_or_zero();
                let sum = v_in + v_out;
                if sum.length_squared() < 0.001 {
                    v_in
                } else {
                    sum.normalize()
                }
            };

            // PARALLEL TRANSPORT:
            // Instead of resetting to `curr.rotation` (which twists), we take our
            // `current_rotation` (from the previous step) and bend it to match the new tangent.
            // This maintains the "Up" vector's relative orientation as much as possible (Bishop Frame).

            let current_forward = current_rotation * Vec3::Y;
            let bend = Quat::from_rotation_arc(current_forward, miter_tangent);

            // Update the running rotation state
            current_rotation = bend * current_rotation;

            ring_start_indices.push(self.add_ring(curr.position, current_rotation, curr.radius));
        }

        // Connect rings
        for i in 0..points_count - 1 {
            self.connect_rings(ring_start_indices[i], ring_start_indices[i + 1]);
        }
    }

    fn add_ring(&mut self, center: Vec3, rotation: Quat, radius: f32) -> u32 {
        let start_index = self.positions.len() as u32;

        for i in 0..=self.resolution {
            let theta = (i as f32 / self.resolution as f32) * std::f32::consts::TAU;
            let (sin, cos) = theta.sin_cos();

            // Ring on XZ plane (Y is forward axis of tube)
            let local_pos = Vec3::new(cos * radius, 0.0, sin * radius);
            let local_normal = Vec3::new(cos, 0.0, sin);

            self.positions.push(center + (rotation * local_pos));
            self.normals.push(rotation * local_normal);
        }

        start_index
    }

    fn connect_rings(&mut self, bottom_start: u32, top_start: u32) {
        for i in 0..self.resolution {
            let bottom_curr = bottom_start + i;
            let bottom_next = bottom_start + i + 1;
            let top_curr = top_start + i;
            let top_next = top_start + i + 1;

            self.indices.push(bottom_curr);
            self.indices.push(top_curr);
            self.indices.push(bottom_next);

            self.indices.push(bottom_next);
            self.indices.push(top_curr);
            self.indices.push(top_next);
        }
    }
}
