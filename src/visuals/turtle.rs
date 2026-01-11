use std::time::Instant;

use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::{SymbolCache, TurtleMaterialHandle, TurtleMeshHandle};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[derive(Component)]
pub struct TurtleSegment;

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

#[derive(Resource, Default)]
pub struct TurtleRenderState {
    pub current_index: usize,
    pub current_turtle: TurtleState,
    pub stack: Vec<TurtleState>,
    pub op_cache: HashMap<u16, TurtleOp>,
    pub is_finished: bool,
    pub total_segments: usize,
    pub processed_count: usize,
}

pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    mesh: Res<TurtleMeshHandle>,
    mat: Res<TurtleMaterialHandle>,
    mut symbol_cache: ResMut<SymbolCache>,
    mut render_state: ResMut<TurtleRenderState>,
    segments: Query<Entity, With<TurtleSegment>>,
) {
    let sys = &engine.0;

    // FIX: Only reset if the ENGINE has changed.
    // `config.is_changed()` is true every frame due to Egui's mutable borrow.
    if engine.is_changed() {
        for entity in &segments {
            commands.entity(entity).despawn();
        }

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

        *render_state = TurtleRenderState {
            current_index: 0,
            current_turtle: TurtleState {
                width: sys.constants.get("width").map(|&w| w as f32).unwrap_or(0.1),
                ..default()
            },
            stack: Vec::with_capacity(64),
            op_cache: op_map,
            is_finished: false,
            total_segments: 0,
            processed_count: 0,
        };
    }

    if render_state.is_finished || sys.state.is_empty() {
        return;
    }

    // 2. Time-Budgeted Loop (8ms)
    let start_time = Instant::now();
    let time_budget = std::time::Duration::from_millis(8);

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

    let mut idx = render_state.current_index;
    let max_len = sys.state.len();

    while idx < max_len {
        if idx % 100 == 0 && start_time.elapsed() > time_budget {
            break;
        }

        let view = match sys.state.get_view(idx) {
            Some(v) => v,
            None => {
                idx = max_len;
                break;
            }
        };

        let op = render_state
            .op_cache
            .get(&view.sym)
            .unwrap_or(&TurtleOp::Ignore);
        let get_val =
            |default: f32| -> f32 { view.params.first().map(|&x| x as f32).unwrap_or(default) };

        match op {
            TurtleOp::Draw => {
                let len = get_val(default_step);
                let t = render_state.current_turtle;

                if len > 0.001 && t.width > 0.001 {
                    let draw_pos = t.transform.translation + t.transform.up() * (len / 2.0);

                    commands.spawn((
                        Mesh3d(mesh.0.clone()),
                        MeshMaterial3d(mat.0.clone()),
                        Transform {
                            translation: draw_pos,
                            rotation: t.transform.rotation,
                            scale: Vec3::new(t.width, len, t.width),
                        },
                        TurtleSegment,
                    ));
                    render_state.total_segments += 1;
                }
                render_state.current_turtle.transform.translation += t.transform.up() * len;
            }
            TurtleOp::Move => {
                let len = get_val(default_step);
                let delta = render_state.current_turtle.transform.up() * len;
                render_state.current_turtle.transform.translation += delta;
            }
            TurtleOp::Yaw(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                render_state.current_turtle.transform.rotate_local_z(angle);
            }
            TurtleOp::Pitch(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                render_state.current_turtle.transform.rotate_local_x(angle);
            }
            TurtleOp::Roll(sign) => {
                let angle = get_val(default_angle.to_degrees()).to_radians() * sign;
                render_state.current_turtle.transform.rotate_local_y(angle);
            }
            TurtleOp::TurnAround => {
                render_state
                    .current_turtle
                    .transform
                    .rotate_local_z(std::f32::consts::PI);
            }
            TurtleOp::Vertical => {
                let h = render_state.current_turtle.transform.up();
                let v = Vec3::Y;
                let l = v.cross(*h).normalize_or_zero();
                if l.length_squared() > 0.001 {
                    let u = h.cross(l).normalize();
                    let rot_matrix = Mat3::from_cols(-l, *h, u);
                    render_state.current_turtle.transform.rotation = Quat::from_mat3(&rot_matrix);
                }
            }
            TurtleOp::SetWidth => {
                let w = get_val(render_state.current_turtle.width);
                render_state.current_turtle.width = w;
            }
            TurtleOp::Push => {
                let t = render_state.current_turtle;
                render_state.stack.push(t);
            }
            TurtleOp::Pop => {
                if let Some(s) = render_state.stack.pop() {
                    render_state.current_turtle = s;
                }
            }
            TurtleOp::Ignore => {}
        }

        idx += 1;
    }

    render_state.current_index = idx;
    render_state.processed_count = idx;

    if render_state.current_index >= sys.state.len() {
        render_state.is_finished = true;
    }
}
