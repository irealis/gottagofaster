use bevy::prelude::*;

/// Contains spawn location and checkpoint location
/// Should be initialized to both being spawn.
/// (checkpoint, spawn)
#[derive(Component)]
pub struct Resetable(pub (Vec3, Vec3));

pub fn reset_pos(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Resetable)>,
) {
    if keyboard_input.just_pressed(KeyCode::Back) {
        for (mut t, res) in &mut query {
            t.translation = res.0 .0;
        }
    }
}
