use std::time::Instant;

use bevy::{ecs::query::Has, prelude::*};

use crate::{character_controller::Grounded, ghost::Ghost, timing::MapDuration};

pub struct AudioPlugin;

#[derive(Resource)]
pub struct SoundsStatus {
    can_play_finish: bool,
    last_grounded_timestamp: Instant,
}

impl Default for SoundsStatus {
    fn default() -> Self {
        SoundsStatus {
            can_play_finish: true,
            last_grounded_timestamp: Instant::now(),
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
    // MapDuration: Only present if the countdown has elapsed and the map is in progress
    mut query: Query<Has<Grounded>, (Added<Grounded>, Without<Ghost>, With<MapDuration>)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for just_grounded in &mut query {
        let time_since_last_grounded = sound_status.last_grounded_timestamp.elapsed().as_millis();

        if just_grounded {
            if time_since_last_grounded > 500 {
                commands.spawn(AudioBundle {
                    source: asset_server.load("landing_sound.ogg"),
                    ..default()
                });
            }
            sound_status.last_grounded_timestamp = Instant::now();
        }
    }
}

/// Reset sounds to allow them to be played again
pub fn reset_sounds(mut sound_status: ResMut<SoundsStatus>) {
    sound_status.can_play_finish = true;
}
