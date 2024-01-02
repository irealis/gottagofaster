use bevy::prelude::*;

use crate::MapEntityMarker;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StateEvents>()
            .add_systems(Update, state_events);
    }
}

#[derive(Event)]
pub enum StateEvents {
    LoadMainscreen,
}

pub fn state_events(
    mut commands: Commands,
    mut er: EventReader<StateEvents>,
    aserv: Res<AssetServer>,
) {
    for e in er.read() {
        match e {
            StateEvents::LoadMainscreen => {
                let mut transform = Transform::from_translation(Vec3::new(25., 5., 30.));
                transform.look_at(Vec3::ZERO, Vec3::Y);
                commands.spawn((
                    Camera3dBundle {
                        transform,
                        ..default()
                    },
                    MapEntityMarker,
                ));

                let map: Handle<Scene> = aserv.load("autumn.glb#Scene0");

                commands.spawn((
                    SceneBundle {
                        scene: map,
                        ..default()
                    },
                    MapEntityMarker,
                ));
            }
        }
    }
}
