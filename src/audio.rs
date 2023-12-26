use bevy::{ecs::query::Has, prelude::*};

use crate::character_controller::JustGrounded;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (play_character_sounds).run_if(in_state(crate::State::Playing)),
        );
    }
}

/// Play sound when the character just touched the ground
pub fn play_character_sounds(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<Has<JustGrounded>>,
) {
    for just_grounded in &mut query {
        if just_grounded {
            commands.spawn(AudioBundle {
                source: asset_server.load("landing_sound.ogg"),
                ..default()
            });
        }
    }
}
