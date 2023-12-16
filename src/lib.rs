mod assets;
mod camera;
mod character_controller;
mod checkpoint;
mod debug;
mod ghost;
mod input;
mod map;
mod physics;
mod ui;
mod vfx;

use std::time::Duration;

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    ecs::system::SystemId,
    math::vec3,
    pbr::{CascadeShadowConfigBuilder, NotShadowCaster},
    prelude::*,
    window::{close_on_esc, PresentMode},
};
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings};
use bevy_hanabi::prelude::*;
use bevy_xpbd_3d::{
    math::{Scalar, Vector},
    prelude::*,
};
use camera::{CameraLeash, LeashedCamera, LeashedCameraBundle, LeashedCameraPlugin};
use character_controller::{CharacterControllerBundle, CharacterControllerPlugin};
use checkpoint::{Checkpoint, Goal};
use ghost::GhostData;
use input::Resetable;
use map::{all_maps, Map};
use physics::PhysicsLayers;
use vfx::{create_ground_effect, create_portal};

use crate::{
    assets::AssetHandles,
    checkpoint::CheckpointPlugin,
    debug::debug_things,
    ghost::GhostPlugin,
    input::reset_pos,
    ui::{setup_ui, ui_finish, ui_mainscreen},
    vfx::{center_sky, emit_ground_effect, VfxPlugin},
};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct Maps {
    maps: Vec<String>,
}

/// Marker for entities that must be unloaded when switching or resetting map.
#[derive(Component)]
pub struct MapEntityMarker;

#[derive(States, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
pub enum State {
    #[default]
    Mainscreen,
    Playing,
    Finished,
}

#[derive(Resource)]
pub struct StateOneshots {
    load_map: SystemId,
    unload: SystemId,
}

pub fn bevy_main() {
    let mut app = App::new();

    #[cfg(not(target_os = "darwin"))]
    let present_mode = PresentMode::Mailbox;
    #[cfg(target_os = "darwin")]
    let present_mode = PresentMode::Fifo;

    app.add_state::<State>()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    ..Default::default()
                }),
            TemporalAntiAliasPlugin,
            PhysicsPlugins::default(),
            CharacterControllerPlugin,
            HanabiPlugin,
            GhostPlugin,
            EguiPlugin,
            FramepacePlugin,
            CheckpointPlugin,
            VfxPlugin,
            //EditorPlugin::default(),
        ))
        .add_plugins(LeashedCameraPlugin)
        .add_systems(Startup, (setup, setup_ui, setup_oneshots))
        .add_systems(
            Update,
            (close_on_esc, ui_mainscreen).run_if(in_state(State::Mainscreen)),
        )
        .add_systems(
            Update,
            (
                debug_things,
                reset_pos,
                setup_scene_once_loaded,
                update_animation,
                rotate_player_model,
            )
                .run_if(in_state(State::Playing)),
        )
        .add_systems(
            Update,
            (close_on_esc, ui_finish).run_if(in_state(State::Finished)),
        );

    dbg!(&app.is_plugin_added::<EguiPlugin>());

    #[cfg(debug_assertions)]
    app.add_plugins(PhysicsDebugPlugin::default());
    //.add_plugins(WorldInspectorPlugin::default());

    app.run();
}

pub fn load_map(
    mut commands: Commands,
    map: Res<Map>,
    asset_handles: Res<AssetHandles>,
    assetserver: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let player_model = assetserver.load("Fox.gltf#Scene0");

    commands.insert_resource(Animations(vec![
        assetserver.load("Fox.gltf#Animation5"), // idle
        assetserver.load("Fox.gltf#Animation3"), // gallop
        assetserver.load("Fox.gltf#Animation4"), // jump
    ]));

    let mut player_transform = Transform::from_translation(map.start_pos);
    player_transform.translation.y -= 1.;
    commands.spawn((
        Name::new("Player"),
        SceneBundle {
            transform: player_transform,
            scene: player_model,
            ..Default::default()
        },
        CameraLeash,
        CharacterControllerBundle::new(
            Collider::compound(vec![(
                Vec3::new(0., 1.5, 0.),
                Quat::default(),
                Collider::ball(1.5),
            )]),
            Vector::NEG_Y * 9.81 * 2.0,
        )
        .with_movement(65.0, 0.98, 10.0, (15.0 as Scalar).to_radians()),
        GhostData::default(),
        Player,
        MapEntityMarker,
        Resetable((map.start_pos, map.start_pos)),
    ));

    commands.spawn((
        LeashedCameraBundle::default(),
        Camera3dBundle {
            transform: Transform::from_xyz(-1.0, 0.1, 1.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            ..Default::default()
        },
        FogSettings {
            color: Color::rgba(0.35, 0.48, 0.66, 1.0),
            directional_light_color: Color::rgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 50.0,
            falloff: FogFalloff::Linear {
                start: 5.,
                end: 600.,
            },
        },
        TemporalAntiAliasBundle::default(),
        MapEntityMarker,
    ));

    let map_data = assetserver.load(format!("{}#Scene0", map.file));
    commands.spawn((
        Name::new("Platform"),
        AsyncSceneCollider::new(Some(ComputedCollider::TriMesh)),
        RigidBody::Static,
        SceneBundle {
            transform: Transform::from_translation(vec3(0., -3., 0.)),
            scene: map_data,
            ..Default::default()
        },
        MapEntityMarker,
        CollisionLayers::new([PhysicsLayers::Ground], [PhysicsLayers::Player]),
    ));

    let portal = effects.add(create_portal());

    commands.spawn((
        Name::new("portal"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(portal),
            transform: Transform::from_translation(map.end_pos)
                .with_scale(Vec3::splat(3.))
                .with_rotation(Quat::from_rotation_y(map.end_rotation.to_radians())),
            ..Default::default()
        },
        Collider::cuboid(10., 10., 3.),
        Goal,
        MapEntityMarker,
    ));

    let ground_effect = effects.add(create_ground_effect());

    let spawner = Spawner::once(100.0.into(), false);

    commands
        .spawn((
            ParticleEffectBundle::new(ground_effect).with_spawner(spawner),
            EffectProperties::default(),
            MapEntityMarker,
        ))
        .insert(Name::new("effect"));

    for checkpoint in &map.checkpoints {
        commands.spawn((
            SceneBundle {
                scene: asset_handles.tori.clone_weak(),
                transform: Transform::from_translation(*checkpoint).with_scale(Vec3::splat(3.)),
                ..Default::default()
            },
            Sensor,
            CollisionLayers::new([PhysicsLayers::Sensor], [PhysicsLayers::Sensor]),
            Collider::cuboid(10., 20., 3.),
            //AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
            RigidBody::Static,
            Checkpoint { reached: false },
            MapEntityMarker,
        ));
    }
}

#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);

pub fn update_animation(
    mut query: Query<(Entity, &LinearVelocity)>,
    mut animation_player: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    children: Query<&Children>,
) {
    for (e, linear_velocity) in &mut query {
        for entity in children.iter_descendants(e) {
            if let Ok(mut animation_player) = animation_player.get_mut(entity) {
                if linear_velocity.0.length() > 2. {
                    animation_player
                        .play_with_transition(
                            animations.0[1].clone_weak(),
                            Duration::from_millis(100),
                        )
                        .repeat();
                } else {
                    animation_player
                        .play_with_transition(
                            animations.0[0].clone_weak(),
                            Duration::from_millis(100),
                        )
                        .repeat();
                }
            }
        }
    }
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

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

pub fn unload(mut commands: Commands, query: Query<Entity, With<MapEntityMarker>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

pub fn setup_oneshots(world: &mut World) {
    let load_map = world.register_system(load_map);
    let unload = world.register_system(unload);

    let oneshots = StateOneshots { load_map, unload };

    world.insert_resource(oneshots);
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pace: ResMut<FramepaceSettings>,
    aserv: Res<AssetServer>,
) {
    let maps = all_maps();
    commands.insert_resource(Maps { maps });

    commands.insert_resource(AssetHandles::load(&aserv));

    //pace.limiter = Limiter::from_framerate(30.);

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 30.,
        maximum_distance: 300.0,
        ..default()
    }
    .build();

    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(0.15, -0.20, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });

    // Sky
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(600.0)),
            ..default()
        },
        NotShadowCaster,
    ));
}
