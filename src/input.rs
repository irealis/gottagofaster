use bevy::prelude::*;
use bevy_xpbd_3d::prelude::LinearVelocity;

use crate::{camera::LeashedCamera, character_controller::JumpCount};

/// Contains spawn location and checkpoint location
/// Should be initialized to both being spawn.
/// (checkpoint, spawn)
#[derive(Component, Default)]
pub struct ResetSnapshot {
    /// Position when going through the checkpoint
    pub pos: Vec3,
    /// linear velocity at the point of the checkpoint
    pub vel: Vec3,
    /// Yaw and pitch of camera at the checkpoint
    pub camera: (f32, f32),
    /// Jump count when passing checkpoint
    pub jump_count: u32,
}

pub fn reset_to_checkpoint(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &mut Transform,
        &mut LinearVelocity,
        &mut JumpCount,
        &ResetSnapshot,
    )>,
    mut camera: Query<&mut LeashedCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::Back) {
        for (mut t, mut lv, mut jc, res) in &mut query {
            t.translation = res.pos;
            lv.0 = res.vel;
            jc.0 = res.jump_count;

            // Would probably be better with events
            for mut cam in &mut camera {
                cam.yaw = res.camera.0;
                cam.pitch = res.camera.1;
            }
        }
    }
}
