use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{CollidingEntities, LinearVelocity};

use crate::Player;

#[derive(Component)]
pub struct Jumppad(pub f32);

pub fn apply_jumppad_boost(
    mut player: Query<(Entity, &mut LinearVelocity), With<Player>>,
    pads: Query<(&CollidingEntities, &Jumppad)>,
) {
    if let Ok((pe, mut linvel)) = player.get_single_mut() {
        for (colliding, pad) in &pads {
            if colliding.contains(&pe) {
                linvel.y += pad.0;
            }
        }
    }
}
