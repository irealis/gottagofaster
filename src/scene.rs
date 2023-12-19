use bevy::prelude::*;

use crate::{assets::Animations, MapEntityMarker};

pub fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

pub fn unload(mut commands: Commands, query: Query<Entity, With<MapEntityMarker>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}
