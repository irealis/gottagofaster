use bevy::{ecs::query::Has, prelude::*};
use instant::Instant;

use crate::{
    character_controller::{GroundEvent, Grounded},
    ghost::Ghost,
    timing::MapDuration,
};

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
    mut commands: Commands,
    mut er: EventReader<GroundEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ in er.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("landing_sound.ogg"),
            ..default()
        });
    }
}

/// Reset sounds to allow them to be played again
pub fn reset_sounds(mut sound_status: ResMut<SoundsStatus>) {
    sound_status.can_play_finish = true;
}
