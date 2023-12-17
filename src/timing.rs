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

#[derive(Component)]
pub struct Countdown(pub Timer);

pub fn tick(time: Res<Time>, mut countdown: Query<&mut Countdown>) {
    for mut cd in &mut countdown {
        cd.0.tick(time.delta());
    }
}

pub fn display_countdown(mut commands: Commands, mut text: Query<(Entity, &Countdown, &mut Text)>) {
    for (e, countdown, mut text) in &mut text {
        if countdown.0.finished() {
            commands.get_entity(e).unwrap().despawn_recursive();
        } else {
            text.sections[0].value = (3. - countdown.0.elapsed().as_secs_f32()).to_string();
            text.sections[0].value.truncate(4);
        }
    }
}

pub fn countdown_timer(
    mut commands: Commands,
    countdown: Query<&Countdown>,
    player: Query<Entity, With<Player>>,
) {
    for cd in &countdown {
        if cd.0.just_finished() {
            if let Ok(player) = player.get_single() {
                commands
                    .get_entity(player)
                    .unwrap()
                    .insert(MapDuration::new());
            }
        }
    }
}
