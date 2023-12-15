use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use bevy::{ecs::system::SystemId, prelude::*};
use bevy_tweening::{
    lens::TransformPositionLens, Animator, EaseMethod, Sequence, Tween, TweeningPlugin,
};
use serde::{Deserialize, Serialize};

use crate::{character_controller::CharacterController, map::Map, MapEntityMarker};

pub struct GhostPlugin;

#[derive(Resource)]
pub struct GhostOneshots {
    pub load: SystemId,
    pub store: SystemId,
}

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .add_systems(Startup, register_oneshots)
            .add_systems(
                Update,
                ghost_recorder.run_if(in_state(crate::State::Playing)),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Default)]
pub struct GhostData {
    log: Vec<Vec3>,
    duration: Vec<f32>,
}

#[derive(Component)]
pub struct GhostDataIndex(usize);

#[derive(Component)]
pub struct Ghost;

#[derive(Component)]
pub struct Replay;

#[derive(Resource)]
pub struct MapName(pub String);

pub fn register_oneshots(world: &mut World) {
    let load = world.register_system(replay_ghost);
    let store = world.register_system(store_ghost);

    world.insert_resource(GhostOneshots { load, store });
}

pub fn store_ghost(
    map: Res<Map>,
    data: Query<&GhostData, Without<Replay>>,
    old_data: Query<&GhostData, With<Replay>>,
) {
    let data = data.single();
    let serialized = serde_json::to_string(data).unwrap();

    if let Ok(old_data) = old_data.get_single() {
        if old_data.duration.iter().sum::<f32>() < data.duration.iter().sum() {
            return;
        }
    }

    // Create or truncate file
    let mut file = File::create(format!("maps/{}.replay", &map.name)).unwrap();
    _ = file.write_all(serialized.as_bytes());
}

pub fn ghost_recorder(
    mut player: Query<(&mut GhostData, &Transform), With<CharacterController>>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
) {
    let (mut ghost_data, transform) = player.single_mut();

    let dt = time.delta_seconds();

    *elapsed += dt;

    if *elapsed > 0.3 {
        ghost_data.log.push(transform.translation);
        ghost_data.duration.push(*elapsed);
        *elapsed = 0.;
    }
}

pub fn replay_ghost(
    map: Res<Map>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let name = format!("maps/{}.replay", &map.name);
    let path = Path::new(&name);
    if !path.exists() {
        return;
    }

    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let data = serde_json::from_str::<GhostData>(&contents).unwrap();

    //let mut tweens;
    let tweens = (0..(data.log.len() - 1)).map(|i| {
        Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(data.duration[i]),
            TransformPositionLens {
                start: data.log[i],
                end: data.log[i + 1],
            },
        )
    });

    let sequence = Sequence::new(tweens);

    let player = meshes.add(Mesh::from(shape::Capsule {
        depth: 2.,
        radius: 1.,
        ..Default::default()
    }));

    commands.spawn((
        Name::new("Ghost"),
        PbrBundle {
            mesh: player,
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0., 0.3, 1., 0.7),
                cull_mode: None,
                ..default()
            }),
            ..Default::default()
        },
        Animator::new(sequence),
        MapEntityMarker,
    ));
}
