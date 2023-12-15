use bevy::{prelude::*, window::CursorGrabMode};
use bevy_xpbd_3d::prelude::{contact_query::intersection_test, Collider, CollidingEntities};

use crate::{ghost::GhostOneshots, Player, State};

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
    mut query: Query<(&CollidingEntities, &mut Checkpoint)>,
    player: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player.get_single() {
        for (colliding, mut checkpoint) in &mut query {
            if colliding.contains(&player) {
                checkpoint.reached = true;
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
    player: Query<(&Collider, &Transform, Has<AllCheckpointsReached>), With<Player>>,
    mut state: ResMut<NextState<State>>,
    mut windows: Query<&mut Window>,
) {
    let (pcollider, ptransform, all_checkpoints_reached) = player.single();

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

                    let mut window = windows.single_mut();
                    window.cursor.grab_mode = CursorGrabMode::None;
                    window.cursor.visible = true;
                }
            }
            Err(_) => panic!("Unsupported intersection shape!"),
        }
    }
}
