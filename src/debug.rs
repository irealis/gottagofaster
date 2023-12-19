use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{
    AsyncSceneCollider, Collider, CollisionLayers, ComputedCollider, RigidBody, Sensor,
};

use crate::{
    assets::AssetHandles, character_controller::MaxSlopeAngle, checkpoint::Checkpoint, load_map,
    map::Map, physics::PhysicsLayers, MapEntityMarker, MapMarker, Player,
};

pub fn debug_things(
    mut player: Query<(&Transform, &mut MaxSlopeAngle), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    aserv: Res<AssetServer>,
    old_map: Res<Map>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    query: Query<Entity, With<Checkpoint>>,
) {
    if keyboard_input.just_pressed(KeyCode::L) {
        if let Ok((player, _)) = player.get_single() {
            dbg!(player);
            println!(
                "[{},{},{}]",
                player.translation.x, player.translation.y, player.translation.z
            );
        }
    }

    if keyboard_input.just_pressed(KeyCode::P) {
        if let Ok((_, mut max)) = player.get_single_mut() {
            max.0 = 180.;
        }
    }

    if keyboard_input.just_pressed(KeyCode::R) {
        aserv.reload(format!("{}.glb#Scene0", old_map.file));

        for e in &query {
            commands.entity(e).despawn_recursive();
        }

        let map = Map::load(&old_map.name);
        for checkpoint in &map.checkpoints {
            commands.spawn((
                SceneBundle {
                    scene: asset_handles.tori.clone_weak(),
                    transform: Transform::from_translation(checkpoint.pos)
                        .with_scale(Vec3::splat(2.))
                        .with_rotation(Quat::from_rotation_y(checkpoint.rot.to_radians())),
                    ..Default::default()
                },
                Sensor,
                CollisionLayers::new([PhysicsLayers::Sensor], [PhysicsLayers::Sensor]),
                Collider::cuboid(10., 20., 3.),
                RigidBody::Static,
                Checkpoint { reached: false },
                MapEntityMarker,
            ));
        }
        commands.insert_resource(map);
        // commands.spawn((
        //     Name::new("Map"),
        //     AsyncSceneCollider::new(Some(ComputedCollider::TriMesh)),
        //     RigidBody::Static,
        //     SceneBundle {
        //         transform: Transform::from_translation(Vec3::new(0., -3., 0.)),
        //         scene: map_data,
        //         ..Default::default()
        //     },
        //     MapEntityMarker,
        //     MapMarker,
        //     CollisionLayers::new([PhysicsLayers::Ground], [PhysicsLayers::Player]),
        // ));
    }
}
