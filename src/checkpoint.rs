use bevy::{prelude::*, window::CursorGrabMode};
use bevy_xpbd_3d::prelude::{
    contact_query::intersection_test, Collider, CollidingEntities, LinearVelocity,
};

use crate::{
    camera::LeashedCamera, character_controller::JumpCount, ghost::GhostOneshots,
    input::ResetSnapshot, leaderboard::LeaderboardEvent, map::Map, timing::MapDuration, Player,
    State,
};

pub struct CheckpointPlugin;

impl Plugin for CheckpointPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_checkpoint, all_checkpoints_reached, on_goal)
                .run_if(in_state(crate::State::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Checkpoint {
    pub reached: bool,
}

#[derive(Component)]
pub struct Goal;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AllCheckpointsReached;

pub fn check_checkpoint(
    mut commands: Commands,
    mut query: Query<(&CollidingEntities, &mut Checkpoint)>,
    player: Query<(Entity, &Transform, &LinearVelocity, &JumpCount), With<Player>>,
    camera: Query<&LeashedCamera>,
) {
    if let Ok((player, transform, vel, jc)) = player.get_single() {
        for (colliding, mut checkpoint) in &mut query {
            if colliding.contains(&player) {
                checkpoint.reached = true;
                let camera = camera.single();

                commands.entity(player).insert(ResetSnapshot {
                    pos: transform.translation,
                    vel: vel.0,
                    camera: (camera.yaw, camera.pitch),
                    jump_count: jc.0,
                });
            }
        }
    }
}

pub fn all_checkpoints_reached(
    mut commands: Commands,
    query: Query<&Checkpoint>,
    player: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player.get_single() {
        if query.iter().all(|checkpoint| checkpoint.reached) {
            commands.entity(player).insert(AllCheckpointsReached);
        }
    }
}

fn on_goal(
    mut commands: Commands,
    oneshots: Res<GhostOneshots>,
    goals: Query<(&Collider, &Transform), With<Goal>>,
    map: Res<Map>,
    mut player: Query<
        (
            &Collider,
            &Transform,
            Has<AllCheckpointsReached>,
            Option<&mut MapDuration>,
        ),
        With<Player>,
    >,
    mut state: ResMut<NextState<State>>,
    mut windows: Query<&mut Window>,
    mut ew: EventWriter<LeaderboardEvent>,
) {
    let (pcollider, ptransform, all_checkpoints_reached, mut mapduration) = player.single_mut();

    for (collider, transform) in &goals {
        match intersection_test(
            pcollider,
            ptransform.translation,
            ptransform.rotation,
            collider,
            transform.translation,
            transform.rotation,
        ) {
            Ok(b) => {
                if b && all_checkpoints_reached {
                    commands.run_system(oneshots.store);

                    state.set(State::Finished);

                    if let Some(ref mut mapduration) = mapduration {
                        mapduration.stop();
                        ew.send(LeaderboardEvent::SaveLeaderboardData(
                            map.name.clone(),
                            mapduration.elapsed().as_secs_f32(),
                        ));
                    }

                    let mut window = windows.single_mut();
                    window.cursor.grab_mode = CursorGrabMode::None;
                    window.cursor.visible = true;
                }
            }
            Err(_) => panic!("Unsupported intersection shape!"),
        }
    }
}
