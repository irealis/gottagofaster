use bevy::{ecs::query::Has, prelude::*};
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use crate::{camera::LeashedCamera, physics::PhysicsLayers, timing::MapDuration};

pub struct CharacterControllerPlugin;

// TODO:
// doublejump doesn't trigger when sliding off because jump count

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    update_grounded,
                    apply_deferred,
                    apply_gravity,
                    movement,
                    apply_movement_damping,
                    decay_multiplier,
                )
                    .chain()
                    .run_if(in_state(crate::State::Playing)),
            )
            .add_systems(
                Update,
                (update_grounded, apply_deferred, apply_gravity)
                    .chain()
                    .run_if(in_state(crate::State::Finished)),
            )
            .add_systems(
                // Run collision handling in substep schedule
                SubstepSchedule,
                kinematic_controller_collisions.in_set(SubstepSet::SolveUserConstraints),
            );
    }
}

#[derive(Event)]
pub enum MovementAction {
    Move(Vector3),
    Jump,
}

#[derive(Component)]
pub struct JumpCount(pub u32);

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Sliding;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

#[derive(Component)]
pub struct AccelerationMultiplier(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// The gravitational acceleration used for a character controller.
#[derive(Component)]
pub struct ControllerGravity(Vector);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(pub Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    gravity: ControllerGravity,
    movement: MovementBundle,
    jump_count: JumpCount,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
    multiplier: AccelerationMultiplier,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
            multiplier: AccelerationMultiplier(1.),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider, gravity: Vector) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Kinematic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Vector::NEG_Y,
            )
            .with_max_time_of_impact(0.2)
            .with_max_hits(2),
            gravity: ControllerGravity(gravity),
            movement: MovementBundle::default(),
            jump_count: JumpCount(0),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let up = keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]);
    let down = keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]);
    let left = keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]);
    let right = keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction =
        Vector3::new(horizontal as Scalar, 0., vertical as Scalar).clamp_length_max(1.0);

    if direction != Vector3::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
    layers: Query<&CollisionLayers>,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let mut is_sliding = false;
        let mut is_grounded = false;
        _ = hits.iter().any(|hit| {
            if let Ok(layer) = layers.get(hit.entity) {
                if !layer.contains_group(PhysicsLayers::Ground) {
                    return false;
                }
            }

            if let Some(angle) = max_slope_angle {
                if rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.0 {
                    is_grounded = true;
                    true
                } else {
                    is_sliding = true;
                    true
                }
            } else {
                true
            }
        });

        if is_grounded {
            // Try: prevent racecondition when unloading
            commands.entity(entity).try_insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }

        if is_sliding {
            commands.entity(entity).try_insert(Sliding);
        } else {
            commands.entity(entity).remove::<Sliding>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<
        (
            &MovementAcceleration,
            &JumpImpulse,
            &mut LinearVelocity,
            &mut JumpCount,
            &mut AccelerationMultiplier,
            Has<Grounded>,
            Has<Sliding>,
        ),
        With<MapDuration>,
    >,
    cameras: Query<&Transform, With<LeashedCamera>>,
) {
    let delta_time = time.delta_seconds();

    let camera_transform = cameras.single();
    for event in movement_event_reader.read() {
        for (
            movement_acceleration,
            jump_impulse,
            mut linear_velocity,
            mut jump_count,
            mut acc_mul,
            is_grounded,
            is_sliding,
        ) in &mut controllers
        {
            match event {
                MovementAction::Move(mut direction) => {
                    direction = camera_transform.rotation.inverse().mul_vec3(direction);
                    linear_velocity.x +=
                        direction.x * movement_acceleration.0 * acc_mul.0 * delta_time;
                    linear_velocity.z -=
                        direction.z * movement_acceleration.0 * acc_mul.0 * delta_time;
                }
                MovementAction::Jump => {
                    if is_grounded || is_sliding {
                        acc_mul.0 += 1.1;
                        dbg!(acc_mul.0);
                        // let forward = camera_transform.rotation.inverse().mul_vec3(Vec3::Z);
                        // linear_velocity.x +=
                        //     forward.x * movement_acceleration.0 * acc_mul.0 * delta_time;
                        // linear_velocity.z -=
                        //     forward.z * movement_acceleration.0 * acc_mul.0 * delta_time;
                        jump_count.0 = 0;
                        linear_velocity.y = jump_impulse.0;
                    } else if jump_count.0 < 2 {
                        jump_count.0 += 1;
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

/// Applies [`ControllerGravity`] to character controllers.
fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    let delta_time = time.delta_seconds();

    for (gravity, mut linear_velocity) in &mut controllers {
        linear_velocity.0 += gravity.0 * delta_time;
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(
    time: Res<Time>,
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>,
) {
    let dt = time.delta_seconds();
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        let factor = damping_factor.0;
        // dbg!(&linear_velocity);
        linear_velocity.x *= factor.powf(dt * 60.);
        linear_velocity.z *= factor.powf(dt * 60.);
        // dbg!(&linear_velocity);
    }
}

/// Slowly decay the acceleration multiplier over time
fn decay_multiplier(
    time: Res<Time>,
    mut query: Query<(&mut AccelerationMultiplier, Has<Grounded>)>,
) {
    let dt = time.delta_seconds();
    for (mut acc, is_grounded) in &mut query {
        let decay_factor = (if is_grounded { 0.97 } else { 0.999 } as f32).powf(dt * 60.);
        acc.0 = (acc.0 * decay_factor).clamp(1.0, 10.);
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system performs very basic collision response for kinematic
/// character controllers by pushing them along their contact normals
/// by the current penetration depths.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, Without<Sensor>>,
    mut character_controllers: Query<
        (
            &RigidBody,
            &mut Position,
            &Rotation,
            &mut LinearVelocity,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
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
        let (rb, mut position, rotation, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(collider_parent1.get()) {
                is_first = true;
                character
            } else if let Ok(character) = character_controllers.get_mut(collider_parent2.get()) {
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
                position.0 += normal * contact.penetration;
            }

            // If the slope isn't too steep to walk on but the character
            // is falling, reset vertical velocity.
            if max_slope_angle.is_some_and(|angle| normal.angle_between(Vector::Y).abs() <= angle.0)
                && linear_velocity.y < 0.0
            {
                linear_velocity.y = linear_velocity.y.max(0.0);
            }
        }
    }
}
