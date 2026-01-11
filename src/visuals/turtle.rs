use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::{SymbolCache, TurtleMaterialHandle, TurtleMeshHandle};
use bevy::prelude::*;

#[derive(Component)]
pub struct TurtleSegment;

#[derive(Clone, Copy)]
struct TurtleState {
    transform: Transform,
    width: f32,
}

impl Default for TurtleState {
    fn default() -> Self {
        Self {
            transform: Transform::IDENTITY,
            width: 0.1, // Default starting width
        }
    }
}

pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    mesh: Res<TurtleMeshHandle>,
    mat: Res<TurtleMaterialHandle>,
    mut symbol_cache: ResMut<SymbolCache>,
    segments: Query<Entity, With<TurtleSegment>>,
) {
    if !config.is_changed() && !engine.is_changed() {
        return;
    }

    // 1. Cleanup
    for entity in &segments {
        commands.entity(entity).despawn();
    }

    let sys = &engine.0;
    if sys.state.is_empty() {
        return;
    }

    symbol_cache.refresh(&sys.interner);

    // 2. Init State
    let mut state = TurtleState::default();
    let mut stack: Vec<TurtleState> = Vec::with_capacity(128);

    // Configuration Priorities: Constant > Config > Default
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

    // Check if initial width is defined in constants
    if let Some(&w) = sys.constants.get("width") {
        state.width = w as f32;
    }

    // 3. Interpretation Loop
    for i in 0..sys.state.len() {
        let view = match sys.state.get_view(i) {
            Some(v) => v,
            None => continue,
        };
        let sym = view.sym;

        let get_val =
            |default: f32| -> f32 { view.params.first().map(|&x| x as f32).unwrap_or(default) };

        if Some(sym) == symbol_cache.f_draw {
            let len = get_val(default_step);

            // Draw Cylinder
            // Move mesh center by +len/2 along local Y (Up)
            let draw_pos = state.transform.translation + state.transform.up() * (len / 2.0);

            commands.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(mat.0.clone()),
                Transform {
                    translation: draw_pos,
                    rotation: state.transform.rotation,
                    // Scale X/Z by width, Y by length
                    scale: Vec3::new(state.width, len, state.width),
                },
                TurtleSegment,
            ));

            // Move Turtle
            state.transform.translation += state.transform.up() * len;
        } else if Some(sym) == symbol_cache.f_move {
            let len = get_val(default_step);
            state.transform.translation += state.transform.up() * len;
        } else if Some(sym) == symbol_cache.yaw_pos {
            state
                .transform
                .rotate_local_z(get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.yaw_neg {
            state
                .transform
                .rotate_local_z(-get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.pitch_pos {
            state
                .transform
                .rotate_local_x(get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.pitch_neg {
            state
                .transform
                .rotate_local_x(-get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.roll_pos {
            state
                .transform
                .rotate_local_y(get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.roll_neg {
            state
                .transform
                .rotate_local_y(-get_val(default_angle.to_degrees()).to_radians());
        } else if Some(sym) == symbol_cache.turn_around {
            state.transform.rotate_local_z(std::f32::consts::PI);
        } else if Some(sym) == symbol_cache.vertical {
            // $: Realign Up vector with World Up to minimize twist
            // Algorithm from "The Algorithmic Beauty of Plants"
            let h = state.transform.up(); // Current Heading (Dir3)
            let v = Vec3::Y; // World Up (Gravity)

            // L = V x H / |V x H|
            // FIX: dereference h (*h) because Vec3::cross expects Vec3, not Dir3
            let l = v.cross(*h).normalize_or_zero();

            // If H and V are parallel, L is zero (singularity). Skip rotation.
            if l.length_squared() > 0.001 {
                // h is Dir3, so it has cross method taking Vec3
                let u = h.cross(l).normalize();

                // Construct new rotation matrix from basis vectors
                // Bevy Basis: Right = -l, Up = h, Back = u
                // FIX: dereference h (*h) because Mat3::from_cols expects Vec3
                let rot_matrix = Mat3::from_cols(-l, *h, u);
                state.transform.rotation = Quat::from_mat3(&rot_matrix);
            }
        } else if Some(sym) == symbol_cache.set_width {
            // !: Set width
            let w = get_val(state.width);
            state.width = w;
        } else if Some(sym) == symbol_cache.push {
            stack.push(state);
        } else if Some(sym) == symbol_cache.pop {
            if let Some(s) = stack.pop() {
                state = s;
            }
        }
    }
}
