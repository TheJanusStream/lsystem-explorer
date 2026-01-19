use bevy::prelude::*;

#[derive(Resource)]
pub struct TurtleMaterialHandle(pub Handle<StandardMaterial>);

pub fn setup_turtle_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(TurtleMaterialHandle(material));
}
