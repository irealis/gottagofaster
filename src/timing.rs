use std::time::{Duration, Instant};

use bevy::prelude::*;

use crate::Player;

#[derive(Component)]
pub struct MapDuration {
    start: Instant,
    end: Option<Instant>,
}

impl MapDuration {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            end: None,
        }
    }

    pub fn stop(&mut self) {
        self.end = Some(Instant::now());
    }

    pub fn elapsed(&self) -> Duration {
        if let Some(end) = self.end {
            end - self.start
        } else {
            Instant::now() - self.start
        }
    }
}

#[derive(Resource)]
pub struct Countdown(pub Timer);

pub fn tick(time: Res<Time>, mut match_time: ResMut<Countdown>) {
    match_time.0.tick(time.delta());
}

pub fn countdown_timer(
    mut commands: Commands,
    countdown: Option<Res<Countdown>>,
    player: Query<Entity, With<Player>>,
) {
    if let Some(countdown) = countdown {
        if countdown.0.just_finished() {
            if let Ok(player) = player.get_single() {
                commands
                    .get_entity(player)
                    .unwrap()
                    .insert(MapDuration::new());
            }
        }
    }
}
