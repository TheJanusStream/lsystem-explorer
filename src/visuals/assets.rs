use bevy::prelude::*;
use symbios::core::interner::SymbolTable;

#[derive(Resource)]
pub struct TurtleMeshHandle(pub Handle<Mesh>);

#[derive(Resource)]
pub struct TurtleMaterialHandle(pub Handle<StandardMaterial>);

/// Maps string symbols to Symbios IDs for fast lookup
#[derive(Resource, Default)]
pub struct SymbolCache {
    pub f_draw: Option<u16>,      // F
    pub f_move: Option<u16>,      // f
    pub yaw_pos: Option<u16>,     // +
    pub yaw_neg: Option<u16>,     // -
    pub pitch_pos: Option<u16>,   // &
    pub pitch_neg: Option<u16>,   // ^
    pub roll_pos: Option<u16>,    // \
    pub roll_neg: Option<u16>,    // /
    pub turn_around: Option<u16>, // |
    pub vertical: Option<u16>,    // $
    pub set_width: Option<u16>,   // !
    pub push: Option<u16>,        // [
    pub pop: Option<u16>,         // ]
}

impl SymbolCache {
    pub fn refresh(&mut self, interner: &SymbolTable) {
        self.f_draw = interner.resolve_id("F");
        self.f_move = interner.resolve_id("f");
        self.yaw_pos = interner.resolve_id("+");
        self.yaw_neg = interner.resolve_id("-");
        self.pitch_pos = interner.resolve_id("&");
        self.pitch_neg = interner.resolve_id("^");
        self.roll_pos = interner.resolve_id("\\");
        self.roll_neg = interner.resolve_id("/");
        self.turn_around = interner.resolve_id("|");
        self.vertical = interner.resolve_id("$");
        self.set_width = interner.resolve_id("!");
        self.push = interner.resolve_id("[");
        self.pop = interner.resolve_id("]");
    }
}

pub fn setup_turtle_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A unit cylinder: height 1.0, radius 0.5 (Diameter 1.0)
    // We use Diameter 1.0 so that scale '0.1' means width '0.1'.
    let mesh = meshes.add(Cylinder::new(0.5, 1.0));
    commands.insert_resource(TurtleMeshHandle(mesh));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(TurtleMaterialHandle(material));

    commands.init_resource::<SymbolCache>();
}