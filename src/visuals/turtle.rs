use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::{SymbolCache, TurtleMaterialHandle};
use crate::visuals::mesher::LSystemMeshBuilder;
use crate::visuals::skeleton::{Skeleton, SkeletonPoint};
use bevy::platform::collections::HashMap;
use bevy::platform::time::Instant;
use bevy::prelude::*;

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

#[derive(Clone, Copy, Debug)]
pub struct StackFrame {
    pub state: TurtleState,
}

#[derive(Resource, Default)]
pub struct TurtleRenderState {
    pub total_vertices: usize,
    pub generation_time_ms: f32,
}

pub fn sync_material_properties(
    config: Res<LSystemConfig>,
    mat_handle: Res<TurtleMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !config.is_changed() {
        return;
    }

    if let Some(mat) = materials.get_mut(&mat_handle.0) {
        mat.base_color = Color::srgb_from_array(config.material_color);
        // Bevy uses LinearRgba for emissive
        let emission_linear =
            Color::srgb_from_array(config.emission_color).to_linear() * config.emission_strength;
        mat.emissive = emission_linear;
    }
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

    let mut skeleton = Skeleton::default();

    let initial_width = sys.constants.get("width").map(|&w| w as f32).unwrap_or(0.1);
    let mut state = TurtleState {
        width: initial_width,
        ..default()
    };

    let mut stack: Vec<StackFrame> = Vec::with_capacity(64);

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

                if skeleton.strands.is_empty() {
                    skeleton.add_node(
                        SkeletonPoint {
                            position: state.transform.translation,
                            rotation: state.transform.rotation,
                            radius: state.width / 2.0,
                        },
                        true,
                    );
                }

                state.transform.translation += state.transform.up() * len;

                if let Some(t_vec) = config.tropism
                    && config.elasticity > 0.0
                {
                    let head = state.transform.up();
                    let h_cross_t = head.cross(t_vec);
                    let mag = h_cross_t.length();
                    if mag > 0.0001
                        && let Ok(axis) = Dir3::new(h_cross_t)
                    {
                        let angle = config.elasticity * mag;
                        state.transform.rotate_axis(axis, angle);
                    }
                }

                let current_point = SkeletonPoint {
                    position: state.transform.translation,
                    rotation: state.transform.rotation,
                    radius: state.width / 2.0,
                };
                skeleton.add_node(current_point, false);
            }
            TurtleOp::Move => {
                let len = get_val(default_step);
                state.transform.translation += state.transform.up() * len;
                skeleton.add_node(
                    SkeletonPoint {
                        position: state.transform.translation,
                        rotation: state.transform.rotation,
                        radius: state.width / 2.0,
                    },
                    true,
                );
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
                stack.push(StackFrame { state });
            }
            TurtleOp::Pop => {
                if let Some(frame) = stack.pop() {
                    state = frame.state;
                    skeleton.add_node(
                        SkeletonPoint {
                            position: state.transform.translation,
                            rotation: state.transform.rotation,
                            radius: state.width / 2.0,
                        },
                        true,
                    );
                }
            }
            TurtleOp::Ignore => {}
        }
    }

    let builder = LSystemMeshBuilder::default();
    let final_mesh = builder.build(&skeleton);

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
