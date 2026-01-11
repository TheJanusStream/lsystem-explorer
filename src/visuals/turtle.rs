use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::{SymbolCache, TurtleMaterialHandle};
use crate::visuals::mesher::LSystemMeshBuilder;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::time::Instant;

#[derive(Component)]
pub struct LSystemMeshTag;

#[derive(Clone, Copy, Debug)]
pub struct TurtleState {
    pub transform: Transform,
    pub width: f32,
}

impl Default for TurtleState {
    fn default() -> Self {
        Self {
            transform: Transform::IDENTITY,
            width: 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TurtleOp {
    Draw,
    Move,
    Yaw(f32),
    Pitch(f32),
    Roll(f32),
    TurnAround,
    Vertical,
    SetWidth,
    Push,
    Pop,
    Ignore,
}

/// Stores state for the Stack (Push/Pop)
#[derive(Clone, Copy, Debug)]
pub struct StackFrame {
    pub state: TurtleState,
    /// The index of the vertex ring at this point in the stack.
    /// Allows branches to connect back to this point.
    pub ring_index: Option<u32>,
}

#[derive(Resource, Default)]
pub struct TurtleRenderState {
    // Metrics only - logic state is now local to the system function
    pub total_vertices: usize,
    pub generation_time_ms: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mat_handle: Res<TurtleMaterialHandle>,
    mut symbol_cache: ResMut<SymbolCache>,
    mut render_state: ResMut<TurtleRenderState>,
    old_meshes: Query<Entity, With<LSystemMeshTag>>,
) {
    let sys = &engine.0;

    if !engine.is_changed() {
        return;
    }

    for entity in &old_meshes {
        commands.entity(entity).despawn();
    }

    if sys.state.is_empty() {
        return;
    }

    let start_time = Instant::now();

    symbol_cache.refresh(&sys.interner);
    let mut op_map = HashMap::new();
    let sc = &*symbol_cache;
    let mut insert = |sym: Option<u16>, op: TurtleOp| {
        if let Some(s) = sym {
            op_map.insert(s, op);
        }
    };

    insert(sc.f_draw, TurtleOp::Draw);
    insert(sc.f_move, TurtleOp::Move);
    insert(sc.yaw_pos, TurtleOp::Yaw(1.0));
    insert(sc.yaw_neg, TurtleOp::Yaw(-1.0));
    insert(sc.pitch_pos, TurtleOp::Pitch(1.0));
    insert(sc.pitch_neg, TurtleOp::Pitch(-1.0));
    insert(sc.roll_pos, TurtleOp::Roll(1.0));
    insert(sc.roll_neg, TurtleOp::Roll(-1.0));
    insert(sc.turn_around, TurtleOp::TurnAround);
    insert(sc.vertical, TurtleOp::Vertical);
    insert(sc.set_width, TurtleOp::SetWidth);
    insert(sc.push, TurtleOp::Push);
    insert(sc.pop, TurtleOp::Pop);

    // Setup Builder
    let mut builder = LSystemMeshBuilder::default();
    let mut state = TurtleState {
        width: sys.constants.get("width").map(|&w| w as f32).unwrap_or(0.1),
        ..default()
    };
    let mut stack: Vec<StackFrame> = Vec::with_capacity(64);

    // We track the index of the "current" ring of vertices.
    // If None, we haven't started a segment chain yet.
    let mut last_ring_idx: Option<u32> = None;

    let default_step = sys
        .constants
        .get("step")
        .map(|&s| s as f32)
        .unwrap_or(config.step_size);
    let default_angle = sys
        .constants
        .get("angle")
        .map(|&a| a as f32)
        .unwrap_or(config.default_angle)
        .to_radians();

    // Loop through ALL modules at once (Fast enough for <500k modules)
    for i in 0..sys.state.len() {
        let view = match sys.state.get_view(i) {
            Some(v) => v,
            None => break,
        };

        let op = op_map.get(&view.sym).unwrap_or(&TurtleOp::Ignore);
        let get_val =
            |default: f32| -> f32 { view.params.first().map(|&x| x as f32).unwrap_or(default) };

        match op {
            TurtleOp::Draw => {
                let len = get_val(default_step);

                // 1. Ensure we have a start ring
                if last_ring_idx.is_none() {
                    last_ring_idx = Some(builder.add_ring(state.transform, state.width / 2.0));
                }

                // 2. Move
                state.transform.translation += state.transform.up() * len;

                // 3. Create End Ring
                let new_ring_idx = builder.add_ring(state.transform, state.width / 2.0);

                // 4. Connect
                if let Some(prev) = last_ring_idx {
                    builder.connect_rings(prev, new_ring_idx);
                }

                // 5. Advance
                last_ring_idx = Some(new_ring_idx);
            }
            TurtleOp::Move => {
                let len = get_val(default_step);
                state.transform.translation += state.transform.up() * len;
                // Move breaks the mesh continuity
                last_ring_idx = None;
            }
            TurtleOp::Yaw(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                state.transform.rotate_local_z(angle);
            }
            TurtleOp::Pitch(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                state.transform.rotate_local_x(angle);
            }
            TurtleOp::Roll(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                state.transform.rotate_local_y(angle);
            }
            TurtleOp::TurnAround => {
                state.transform.rotate_local_z(std::f32::consts::PI);
            }
            TurtleOp::Vertical => {
                let h = state.transform.up();
                let v = Vec3::Y;
                let l = v.cross(*h).normalize_or_zero();
                if l.length_squared() > 0.001 {
                    let u = h.cross(l).normalize();
                    let rot_matrix = Mat3::from_cols(-l, *h, u);
                    state.transform.rotation = Quat::from_mat3(&rot_matrix);
                }
            }
            TurtleOp::SetWidth => {
                state.width = get_val(state.width);
            }
            TurtleOp::Push => {
                // Save current state AND the current ring index
                // This allows the branch to attach to the current ring
                stack.push(StackFrame {
                    state,
                    ring_index: last_ring_idx,
                });
            }
            TurtleOp::Pop => {
                if let Some(frame) = stack.pop() {
                    state = frame.state;
                    // Restore the ring index.
                    // This means the next Draw command will connect back to where we pushed.
                    last_ring_idx = frame.ring_index;
                }
            }
            TurtleOp::Ignore => {}
        }
    }

    let final_mesh = builder.build();
    render_state.total_vertices = final_mesh.count_vertices();
    let mesh_handle = meshes.add(final_mesh);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(mat_handle.0.clone()),
        Transform::IDENTITY,
        LSystemMeshTag,
    ));

    render_state.generation_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}
