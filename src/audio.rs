use std::time::SystemTime;

use bevy::{ecs::query::Has, prelude::*};

use crate::character_controller::JustGrounded;

pub struct AudioPlugin;

#[derive(Resource)]
pub struct SoundsStatus {
    can_play_finish: bool,
    last_grounded_timestamp: SystemTime,
}

impl Default for SoundsStatus {
    fn default() -> Self {
        SoundsStatus {
            can_play_finish: true,
            last_grounded_timestamp: SystemTime::now(),
        }
    }
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoundsStatus::default())
            .add_systems(
                Update,
                (reset_sounds, play_character_sounds).run_if(in_state(crate::State::Playing)),
            )
            .add_systems(
                Update,
                (play_finish_sounds).run_if(in_state(crate::State::Finished)),
            );
    }
}

/// Play sound when the character just touched the ground
pub fn play_finish_sounds(
    mut sound_status: ResMut<SoundsStatus>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if sound_status.can_play_finish {
        commands.spawn(AudioBundle {
            source: asset_server.load("finish_sound.ogg"),
            ..default()
        });
        sound_status.can_play_finish = false;
    }
}

/// Play sound when the character just touched the ground
pub fn play_character_sounds(
    mut sound_status: ResMut<SoundsStatus>,
    mut query: Query<Has<JustGrounded>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for just_grounded in &mut query {
        let time_since_last_grounded = sound_status
            .last_grounded_timestamp
            .elapsed()
            .expect("system time cannot be in the past")
            .as_millis();

        if just_grounded {
            if time_since_last_grounded > 500 {
                commands.spawn(AudioBundle {
                    source: asset_server.load("landing_sound.ogg"),
                    ..default()
                });
            }
            sound_status.last_grounded_timestamp = SystemTime::now();
        }
    }
}

/// Reset sounds to allow them to be played again
pub fn reset_sounds(mut sound_status: ResMut<SoundsStatus>) {
    sound_status.can_play_finish = true;
}
