use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SkeletonPoint {
    pub position: Vec3,
    pub rotation: Quat,
    pub radius: f32,
}

#[derive(Default, Resource)]
pub struct Skeleton {
    pub strands: Vec<Vec<SkeletonPoint>>,
}

impl Skeleton {
    /// Adds a point to the current strand, or starts a new one if `new_strand` is true
    pub fn add_node(&mut self, point: SkeletonPoint, force_new_strand: bool) {
        if force_new_strand || self.strands.is_empty() {
            self.strands.push(vec![point]);
        } else if let Some(last_strand) = self.strands.last_mut() {
            // Optimization: Don't add duplicate points (zero length segments)
            if let Some(last_point) = last_strand.last()
                && last_point.position.distance_squared(point.position) < 0.00001
            {
                return;
            }
            last_strand.push(point);
        }
    }
}
