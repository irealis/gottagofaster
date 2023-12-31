use std::{fs::File, io::Read, path::Path};

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{
    AsyncSceneCollider, CollisionLayers, ComputedCollider, RigidBody, VHACDParameters,
};
use serde::{Deserialize, Serialize};

use crate::{physics::PhysicsLayers, MapEntityMarker, MapMarker};

#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub file: String,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub end_rotation: f32,
    pub start_rotation: f32,
    pub checkpoints: Vec<Checkpoint>,
    pub pads: Option<Vec<Jumppad>>,
    collidertype: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub pos: Vec3,
    pub rot: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jumppad {
    pub pos: Vec3,
    pub strength: f32,
}

impl From<&str> for Map {
    fn from(value: &str) -> Self {
        Map::load(value)
    }
}

impl Map {
    pub fn load(name: &str) -> Self {
        let path = Path::new("maps").join(name);
        if !path.exists() {
            panic!("Map doesn't exist.");
        }

        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str::<Map>(&contents).expect("Map could not be load from json")
    }

    pub fn collider_type(&self) -> ComputedCollider {
        match self.collidertype {
            Some(0) => ComputedCollider::ConvexDecomposition(VHACDParameters::default()),
            Some(1) => ComputedCollider::ConvexHull,
            Some(2) => ComputedCollider::TriMesh,
            _ => ComputedCollider::ConvexDecomposition(VHACDParameters::default()),
        }
    }
}

pub fn all_maps() -> Vec<String> {
    let path = Path::new("maps");
    let dir = path.read_dir().unwrap();

    dir.map(|f| {
        println!("{:?}", f.as_ref().unwrap());
        // I love you rust <3
        f.unwrap().file_name().to_str().unwrap().into()
    })
    .filter(|m: &String| !m.ends_with("replay"))
    .collect()
}

pub fn spawn_map(
    assetserver: Res<'_, AssetServer>,
    map: &Res<'_, Map>,
    commands: &mut Commands<'_, '_>,
) {
    let map_data = assetserver.load(format!("{}.glb#Scene0", map.file));
    commands.spawn((
        Name::new("Map"),
        AsyncSceneCollider::new(Some(ComputedCollider::TriMesh)),
        RigidBody::Static,
        SceneBundle {
            transform: Transform::from_translation(Vec3::new(0., -3., 0.)),
            scene: map_data,
            ..Default::default()
        },
        MapEntityMarker,
        MapMarker,
        CollisionLayers::new([PhysicsLayers::Ground], [PhysicsLayers::Player]),
    ));
}
