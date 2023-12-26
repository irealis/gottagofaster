use std::time::Duration;

use bevy::prelude::*;
use bevy_xpbd_3d::{
    math::Scalar,
    prelude::{Collider, LinearVelocity, RayCaster},
};

use crate::{
    assets::{Animations, AssetHandles},
    camera::{CameraLeash, LeashedCamera},
    character_controller::{CharacterControllerBundle, Grounded, JumpCount, Sliding},
    ghost::GhostData,
    input::ResetSnapshot,
    map::Map,
    MapEntityMarker, Player,
};

pub fn spawn_player(map: &Res<Map>, commands: &mut Commands, asset_handles: &Res<AssetHandles>) {
    let mut player_transform = Transform::from_translation(map.start_pos);
    player_transform.translation.y -= 1.;

    commands.spawn((
        Name::new("Player"),
        SceneBundle {
            transform: player_transform,
            scene: asset_handles.fox.clone(),
            ..Default::default()
        },
        CameraLeash,
        CharacterControllerBundle::new(
            Collider::compound(vec![(
                Vec3::new(0., 1.5, 0.),
                Quat::default(),
                Collider::ball(1.5),
            )]),
            Vec3::NEG_Y * 9.81 * 2.0,
        )
        .with_movement(30.0, 0.98, 10.0, (15.0 as Scalar).to_radians()),
        GhostData::default(),
        Player,
        MapEntityMarker,
        ResetSnapshot::default(),
        RayCaster::new(Vec3::new(0., 1., 0.), Vec3::ZERO).with_max_hits(1),
    ));
}

pub fn rotate_player_model(
    mut query: Query<&mut Transform, With<Player>>,
    cameras: Query<&Transform, (With<LeashedCamera>, Without<Player>)>,
) {
    let camera_transform = cameras.single();
    let forward = camera_transform
        .rotation
        .inverse()
        .mul_vec3(Vec3::Z)
        .normalize();
    for mut transform in &mut query {
        let angle = forward.xz().angle_between(Vec2::NEG_Y);

        transform.rotation = Quat::from_rotation_y(-angle);
    }
}

pub fn update_player_animation(
    mut query: Query<(
        Entity,
        &LinearVelocity,
        Has<Grounded>,
        Has<Sliding>,
        &JumpCount,
    )>,
    mut animation_player: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    children: Query<&Children>,
    mut jc: Local<i32>,
) {
    for (e, linear_velocity, is_grounded, is_sliding, jump_count) in &mut query {
        for entity in children.iter_descendants(e) {
            if let Ok(mut animation_player) = animation_player.get_mut(entity) {
                if linear_velocity.0.length() > 2. && (is_sliding || is_grounded) {
                    animation_player
                        .play_with_transition(
                            animations.0[1].clone_weak(),
                            Duration::from_millis(100),
                        )
                        .repeat();
                    *jc = -1;
                } else if !is_grounded && !is_sliding {
                    if *jc != jump_count.0 as i32 {
                        if animation_player.is_playing_clip(&animations.0[2]) {
                            animation_player.replay();
                        } else {
                            animation_player.play_with_transition(
                                animations.0[2].clone_weak(),
                                Duration::from_millis(50),
                            );
                        }
                        *jc = jump_count.0 as i32;
                    }
                } else {
                    animation_player
                        .play_with_transition(
                            animations.0[0].clone_weak(),
                            Duration::from_millis(100),
                        )
                        .repeat();
                    *jc = -1;
                }
            }
        }
    }
}
