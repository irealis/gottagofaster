use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion, math::vec3, prelude::*, transform::TransformSystem,
    window::CursorGrabMode,
};
use bevy_xpbd_3d::{prelude::*, PhysicsSet};

pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

#[derive(Bundle, Debug)]
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
    pitch: f32,
    yaw: f32,
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
            (
                leash_camera,
                toggle_camera_lock,
                //camera_collisions.after(leash_camera),
            )
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate)
                .run_if(in_state(crate::State::Playing)),
        );
    }
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
    view: Query<&Transform, (With<CameraLeash>, Without<Camera3d>)>,
    mut cameras: Query<
        (
            &mut Transform,
            &mut LeashedCamera,
            &IgnoreMouseInput,
            &CameraDistance,
        ),
        With<Camera3d>,
    >,
    mut mouse_events: EventReader<MouseMotion>,
) {
    let leash = view.single();

    let mut leash_translation_offset = leash.translation;
    leash_translation_offset.y += 1.5;

    let mut mouse_delta = Vec2::ZERO;
    for mouse_event in mouse_events.read() {
        mouse_delta += mouse_event.delta;
    }

    for (mut transform, mut camera, ignore_mouse, distance) in &mut cameras {
        if ignore_mouse.0 {
            continue;
        }

        let sensitivity = 0.1;
        camera.pitch =
            (camera.pitch - mouse_delta.y * RADIANS_PER_DOT * sensitivity).clamp(-PI / 2., PI / 2.);
        camera.yaw -= mouse_delta.x * RADIANS_PER_DOT * sensitivity;

        let rot = Quat::from_euler(EulerRot::YXZ, camera.yaw, -camera.pitch, 0.);
        transform.translation = leash_translation_offset + rot * vec3(0., 0., -distance.0);
        transform.look_at(leash_translation_offset, Vec3::Y);
    }
}

#[allow(clippy::type_complexity)]
fn camera_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, Without<Sensor>>,
    mut view: Query<(&RigidBody, &Rotation, &mut CameraDistance), With<LeashedCamera>>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }

        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([collider_parent1, collider_parent2]) =
            collider_parents.get_many([contacts.entity1, contacts.entity2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;
        let (rb, rotation, mut distance) =
            if let Ok(character) = view.get_mut(collider_parent1.get()) {
                is_first = true;
                character
            } else if let Ok(character) = view.get_mut(collider_parent2.get()) {
                is_first = false;
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers
        if !rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.global_normal1(rotation)
            } else {
                -manifold.global_normal2(rotation)
            };

            // Solve each penetrating contact in the manifold
            for contact in manifold.contacts.iter().filter(|c| c.penetration > 0.0) {
                let length = (normal * contact.penetration).length();
                distance.0 = (distance.0 - (length + 0.3)).clamp(2., 15.);
            }
        }
    }
}
