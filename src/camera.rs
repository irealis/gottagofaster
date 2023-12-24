use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, experimental::taa::TemporalAntiAliasBundle, tonemapping::Tonemapping,
    },
    input::mouse::MouseMotion,
    math::vec3,
    pbr::ShadowFilteringMethod,
    prelude::*,
    transform::TransformSystem,
    window::CursorGrabMode,
};
use bevy_xpbd_3d::{prelude::*, PhysicsSet};

use crate::{MapEntityMarker, Player};

pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

#[derive(Bundle)]
pub struct LeashedCameraBundle {
    pub camera: LeashedCamera,
    pub ignore_mouse: IgnoreMouseInput,
    pub collider: Collider,
    pub body: RigidBody,
    pub distance: CameraDistance,
}

impl Default for LeashedCameraBundle {
    fn default() -> Self {
        Self {
            camera: LeashedCamera { pitch: 0., yaw: 0. },
            ignore_mouse: IgnoreMouseInput(false),
            collider: Collider::ball(0.1),
            body: RigidBody::Kinematic,
            distance: CameraDistance(30.),
        }
    }
}

impl From<bool> for IgnoreMouseInput {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

#[derive(Component, Debug)]
pub struct CameraDistance(f32);

#[derive(Component)]
pub struct CameraLeash;

#[derive(Component, Reflect, Debug)]
pub struct LeashedCamera {
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for LeashedCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

pub struct LeashedCameraPlugin;

#[derive(Component, Debug, Reflect)]
pub struct IgnoreMouseInput(pub bool);

impl Plugin for LeashedCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (leash_camera, toggle_camera_lock, raycast_camera)
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate)
                .run_if(in_state(crate::State::Playing)),
        );
    }
}

pub fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        LeashedCameraBundle::default(),
        Camera3dBundle {
            transform: Transform::from_xyz(-1.0, 0.1, 1.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        ShadowFilteringMethod::Jimenez14,
        BloomSettings::default(),
        FogSettings {
            color: Color::hex("bd6868ff").unwrap(),
            directional_light_color: Color::rgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 50.0,
            falloff: FogFalloff::Linear {
                start: 5.,
                end: 400.,
            },
        },
        TemporalAntiAliasBundle::default(),
        MapEntityMarker,
    ));
}

fn toggle_camera_lock(
    key_input: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut cameras: Query<&mut IgnoreMouseInput, With<LeashedCamera>>,
) {
    let mut window = windows.single_mut();
    if key_input.just_pressed(KeyCode::Comma) {
        if !window.cursor.visible || !window.focused {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        } else {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        }

        for mut ignore in &mut cameras {
            ignore.0 = !ignore.0;
        }
    }
}

fn leash_camera(
    mut player: Query<(&mut RayCaster, &Transform), (With<CameraLeash>, Without<Camera3d>)>,
    mut cameras: Query<(&mut LeashedCamera, &IgnoreMouseInput, &CameraDistance), With<Camera3d>>,
    mut mouse_events: EventReader<MouseMotion>,
) {
    if let Ok(player) = player.get_single_mut() {
        let (mut raycaster, leash_transform) = player;

        let mut leash_translation_offset = leash_transform.translation;
        leash_translation_offset.y += 1.5;

        let mut mouse_delta = Vec2::ZERO;
        for mouse_event in mouse_events.read() {
            mouse_delta += mouse_event.delta;
        }

        for (mut camera, ignore_mouse, distance) in &mut cameras {
            if ignore_mouse.0 {
                continue;
            }

            let sensitivity = 0.1;
            camera.pitch = (camera.pitch - mouse_delta.y * RADIANS_PER_DOT * sensitivity)
                .clamp(-PI / 2., PI / 2.);
            camera.yaw -= mouse_delta.x * RADIANS_PER_DOT * sensitivity;

            raycaster.direction = Quat::from_rotation_x(-camera.pitch) * vec3(0., 0., -distance.0);
        }
    }
}

fn raycast_camera(
    player: Query<(&Transform, Option<&RayHits>), (With<RayCaster>, With<Player>)>,
    has_sensor: Query<Has<Sensor>>,
    mut camera: Query<(&mut Transform, &CameraDistance, &LeashedCamera), Without<Player>>,
) {
    for (transform, hits) in &player {
        if let Ok((mut camera, distance, leashed_camera)) = camera.get_single_mut() {
            let mut dist = 1.0;
            if let Some(hits) = hits {
                for hit in hits.iter_sorted() {
                    if has_sensor.get(hit.entity).unwrap_or(false) {
                        continue;
                    }
                    dist = hit.time_of_impact.clamp(0.0, 1.0);
                    break;
                }
            }

            let rot =
                Quat::from_euler(EulerRot::YXZ, leashed_camera.yaw, -leashed_camera.pitch, 0.);
            camera.translation = transform.translation + rot * vec3(0., 0., -(distance.0 * dist));
            camera.look_at(transform.translation, Vec3::Y);
        }
    }
}
