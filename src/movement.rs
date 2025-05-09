use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    camera::MainCamera,
    character::*,
    input::{self, DefaultContext, Jump},
    move_and_slide::*,
};

// @todo: we should probably move all of this into an example file, then make the project a lib instead of a bin.

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, movement);
    }
}

#[derive(Component)]
#[require(
    RigidBody = RigidBody::Kinematic,
    Collider = Capsule3d::new(EXAMPLE_CHARACTER_RADIUS, EXAMPLE_CHARACTER_CAPSULE_LENGTH),
)]
pub struct Character {
    velocity: Vec3,
    floor: Option<Dir3>,
    up: Dir3,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            floor: None,
            up: Dir3::Y,
        }
    }
}

// Marker component used to freeze player movement when the main camera is in fly-mode.
// This shouldn't be strictly necessary if we figure out how to properly layer InputContexts.
#[derive(Component)]
pub struct Frozen;

fn movement(
    mut q_kcc: Query<
        (
            Entity,
            &Actions<DefaultContext>,
            &mut Transform,
            &mut Character,
            &Collider,
            &CollisionLayers,
        ),
        Without<Frozen>,
    >,
    main_camera: Single<&Transform, (With<MainCamera>, Without<Character>)>,
    sensors: Query<Entity, With<Sensor>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    let main_camera_transform = main_camera.into_inner();
    for (entity, actions, mut transform, mut character, collider, layers) in &mut q_kcc {
        if actions.action::<Jump>().state() == ActionState::Fired {
            if character.floor.take().is_some() {
                // Override downard velocity
                let down_vel = character.velocity.dot(*character.up).min(0.0);
                let jump_impulse = character.up * (EXAMPLE_JUMP_IMPULSE - down_vel);
                character.velocity += jump_impulse;
            }
        }

        // Get the raw 2D input vector
        let input_vec = actions.action::<input::Move>().value().as_axis2d();

        // Extract just the yaw from the camera rotation
        let camera_yaw = main_camera_transform.rotation.to_euler(EulerRot::YXZ).0;
        let yaw_rotation = Quat::from_rotation_y(camera_yaw);

        // Rotate the movement direction vector by only the camera's yaw
        let direction = yaw_rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y);

        let max_acceleration = match character.floor {
            Some(_) => {
                let friction = friction(character.velocity, EXAMPLE_FRICTION, time.delta_secs());
                character.velocity += friction;

                EXAMPLE_FLOOR_ACCELERATION
            }
            None => {
                // Apply gravity when not grounded
                let gravity = character.up * -EXAMPLE_GRAVITY * time.delta_secs();
                character.velocity += gravity;

                EXAMPLE_AIR_ACCELERATION
            }
        };

        // accelerate in the movement direction
        let acceleration = acceleration(
            character.velocity,
            direction,
            max_acceleration,
            EXAMPLE_MOVEMENT_SPEED,
            time.delta_secs(),
        );
        character.velocity += acceleration;

        let rotation = transform.rotation;

        // Filter out the character entity as well as any entities not in the character's collision filter
        let mut filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity])
            .with_mask(layers.filters);

        // Also filter out sensor entities
        filter.excluded_entities.extend(sensors);

        let config = MoveAndSlideConfig::default();

        // We need to store the new ground for the ground check to work properly
        let mut new_floor = None;

        if let Some(move_and_slide_result) = move_and_slide(
            &spatial_query,
            &collider,
            transform.translation,
            character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |movement| {
                let walkable = is_walkable(
                    movement.hit_data.normal1,
                    character.up,
                    EXAMPLE_WALKABLE_ANGLE,
                );

                if walkable {
                    // Dir3::new won't be Err since we have already checked if it's walkable
                    new_floor = Some(Dir3::new(movement.hit_data.normal1).unwrap());
                }

                let grounded = character.floor.is_some() || new_floor.is_some();

                // In order to try step up we need to be grounded and hitting a "wall".
                if walkable || !grounded {
                    return true;
                }

                let horizontal_normal = movement
                    .hit_data
                    .normal1
                    .reject_from_normalized(*character.up)
                    .normalize_or_zero();

                // This is necessary for capsule colliders since the normal angle changes depending on
                // how far out on a ledge the character is standing
                let a = 1.0 - EXAMPLE_WALKABLE_ANGLE.cos();
                let min_inward_distance = EXAMPLE_CHARACTER_RADIUS * a;

                // Step into the hit normal alil bit, this helps with the capsule collider.
                // Cylinders don't need this since they have a flat bottom.
                let inward = min_inward_distance + config.epsilon * PI;

                // Step a lil bit less forward to account for stepping into the hit normal
                let forward = (movement.remaining_motion - inward).max(0.0);

                let step_motion = movement.direction * forward - horizontal_normal * inward;

                let Some((step_offset, step_hit)) = try_climb_step(
                    &spatial_query,
                    &collider,
                    *movement.translation,
                    step_motion,
                    rotation,
                    character.up,
                    EXAMPLE_STEP_HEIGHT + EXAMPLE_FLOOR_CHECK_DISTANCE,
                    config.epsilon,
                    &filter,
                ) else {
                    // Can't stand here, slide instead
                    return true;
                };

                if !is_walkable(step_hit.normal1, character.up, EXAMPLE_WALKABLE_ANGLE) {
                    return true;
                }

                // Make sure velocity is not upwards after stepping. This is because if 
                // we're a capsule, the roundness of it will cause an upward velocity, 
                // giving us a launching up effect that we don't want.
                let up_vel = movement.translation.dot(*character.up).max(0.0);
                *movement.velocity -= character.up * up_vel;

                // We need to override the translation here because the we stepped up
                *movement.translation = step_offset;

                new_floor = Some(Dir3::new(step_hit.normal1).unwrap());

                // Subtract the stepped distance from remaining time to avoid moving further
                let move_time = (forward + inward) * time.delta_secs();
                *movement.remaining_time = (*movement.remaining_time - move_time).max(0.0);

                // Successfully stepped, don't slide this iteration
                false
            },
        ) {
            transform.translation = move_and_slide_result.new_translation;
            character.velocity = move_and_slide_result.new_velocity;
        }

        if character.floor.is_some() && new_floor.is_none() {
            if let Some((movement, hit)) = floor_check(
                &collider,
                &config,
                transform.translation,
                character.up,
                rotation,
                &spatial_query,
                &filter,
                EXAMPLE_FLOOR_CHECK_DISTANCE,
                EXAMPLE_WALKABLE_ANGLE,
            ) {
                transform.translation -= movement * character.up;
                new_floor = Some(Dir3::new(hit.normal1).unwrap());
            }
        }

        character.floor = new_floor;
    }
}

/// This is a simple example inspired by Quake, users are expected to bring their own logic for acceleration.
#[must_use]
fn acceleration(
    velocity: Vec3,
    direction: impl TryInto<Dir3>,
    max_acceleration: f32,
    target_speed: f32,
    delta: f32,
) -> Vec3 {
    let Ok(direction) = direction.try_into() else {
        return Vec3::ZERO;
    };

    // Current speed in the desired direction.
    let current_speed = velocity.dot(*direction);

    // No acceleration is needed if current speed exceeds target.
    if current_speed >= target_speed {
        return Vec3::ZERO;
    }

    // Clamp to avoid acceleration past the target speed.
    let accel_speed = f32::min(target_speed - current_speed, max_acceleration * delta);

    direction * accel_speed
}

/// Constant acceleration in the opposite direction of velocity.
#[must_use]
pub fn friction(velocity: Vec3, friction: f32, delta: f32) -> Vec3 {
    let speed_sq = velocity.length_squared();

    if speed_sq < 1e-4 {
        return Vec3::ZERO;
    }

    let factor = f32::exp(-friction / speed_sq.sqrt() * delta);

    -velocity * (1.0 - factor)
}
