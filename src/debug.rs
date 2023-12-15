use bevy::prelude::*;

use crate::Player;

pub fn debug_things(player: Query<&Transform, With<Player>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::L) {
        if let Ok(player) = player.get_single() {
            dbg!(player);
        }
    }
}
