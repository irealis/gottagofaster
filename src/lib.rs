mod assets;
mod audio;
mod camera;
mod character_controller;
mod checkpoint;
mod debug;
mod environment;
mod ghost;
mod input;
mod jumppad;
mod map;
mod physics;
mod player;
mod scene;
mod timing;
mod ui;
mod vfx;

use std::time::Duration;

use assets::Animations;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
    ecs::system::SystemId,
    math::vec3,
    prelude::*,
    window::{close_on_esc, PresentMode},
};
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_hanabi::prelude::*;
use bevy_xpbd_3d::prelude::*;
use camera::{spawn_camera, LeashedCameraPlugin};
use character_controller::CharacterControllerPlugin;
use checkpoint::{Checkpoint, Goal};
use environment::spawn_sky;
use jumppad::Jumppad;
use map::{all_maps, spawn_map, Map};
use physics::PhysicsLayers;
use player::{rotate_player_model, spawn_player, update_player_animation};
use scene::{setup_scene_once_loaded, unload};
use timing::Countdown;
use ui::{spawn_countdown_display, to_main_menu};
use vfx::{create_ground_effect, create_portal};

use crate::{
    assets::AssetHandles,
    audio::AudioPlugin,
    checkpoint::CheckpointPlugin,
    debug::debug_things,
    ghost::GhostPlugin,
    input::reset_to_checkpoint,
    jumppad::apply_jumppad_boost,
    timing::{countdown_timer, display_countdown, tick},
    ui::{setup_ui, ui_finish, ui_mainscreen},
    vfx::VfxPlugin,
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

#[derive(Component)]
pub struct MapMarker;

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

    #[cfg(not(target_os = "macos"))]
    let present_mode = PresentMode::Mailbox;
    #[cfg(target_os = "macos")]
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
                    mode: AssetMode::Unprocessed,
                    ..Default::default()
                }),
            TemporalAntiAliasPlugin,
            PhysicsPlugins::default(),
            CharacterControllerPlugin,
            AudioPlugin,
            HanabiPlugin,
            GhostPlugin,
            EguiPlugin,
            //FramepacePlugin,
            CheckpointPlugin,
            VfxPlugin,
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
                reset_to_checkpoint,
                setup_scene_once_loaded,
                update_player_animation,
                rotate_player_model,
                apply_jumppad_boost,
                countdown_timer,
                tick,
                display_countdown,
                to_main_menu,
            )
                .run_if(in_state(State::Playing)),
        )
        .add_systems(
            Update,
            (close_on_esc, ui_finish).run_if(in_state(State::Finished)),
        );

    // Note: Enabling debug visualization has a big performance hit
    #[cfg(feature = "physics_debug")]
    app.add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(PhysicsDebugConfig {
            aabb_color: Some(Color::ANTIQUE_WHITE),
            ..default()
        });

    app.run();
}

pub fn load_map(
    mut commands: Commands,
    map: Res<Map>,
    asset_handles: Res<AssetHandles>,
    assetserver: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    commands.insert_resource(Animations(vec![
        assetserver.load("Fox.gltf#Animation5"), // idle
        assetserver.load("Fox.gltf#Animation3"), // gallop
        assetserver.load("Fox.gltf#Animation4"), // jump
    ]));

    spawn_player(&map, &mut commands, &asset_handles);

    spawn_camera(&mut commands, map.start_rotation.to_radians());

    spawn_map(assetserver, &map, &mut commands);
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

    if let Some(pads) = &map.pads {
        for pad in pads {
            commands.spawn((
                SceneBundle {
                    scene: asset_handles.pad.clone_weak(),
                    transform: Transform::from_translation(pad.pos),
                    ..Default::default()
                },
                Collider::cylinder(0.6, 4.8),
                Sensor,
                RigidBody::Static,
                MapEntityMarker,
                Jumppad(pad.strength),
            ));
        }
    }

    spawn_countdown_display(commands);
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
    //mut pace: ResMut<FramepaceSettings>,
    aserv: Res<AssetServer>,
) {
    let maps = all_maps();
    commands.insert_resource(Maps { maps });

    commands.insert_resource(AssetHandles::load(&aserv));

    //pace.limiter = Limiter::from_framerate(30.);
    commands.insert_resource(Time::new_with(Physics::variable(1. / 30.)));

    spawn_sky(commands, &mut meshes, &mut materials);
}
