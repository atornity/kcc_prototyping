use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use kcc_prototype::move_and_slide::{MoveAndSlideConfig, character_sweep, move_and_slide};

use crate::input::{DefaultContext, Jump, Move};

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, movement);
    }
}

#[derive(Component)]
#[require(
    RigidBody = RigidBody::Kinematic,
    Collider = Capsule3d::new(0.35, 1.0),
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

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
const EXAMPLE_FLOOR_ACCELERATION: f32 = 100.0;
const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
const EXAMPLE_FRICTION: f32 = 60.0;
const EXAMPLE_WALKABLE_ANGLE: f32 = 30.0_f32.to_radians();
const EXAMPLE_JUMP_IMPULSE: f32 = 8.0;
const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games

fn movement(
    mut q_kcc: Query<(
        Entity,
        &mut Transform,
        &mut Character,
        &Collider,
        &CollisionLayers,
    )>,
    q_input: Single<&Actions<DefaultContext>>,
    sensors: Query<Entity, With<Sensor>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut transform, mut character, collider, layers) in &mut q_kcc {
        // FIXME: this fires every frame [space] is held down, it should only trigger when initially pressed
        if q_input.action::<Jump>().state() == ActionState::Fired {
            println!("Jump action fired!");
            if character.floor.is_some() {
                let impulse = character.up * EXAMPLE_JUMP_IMPULSE;
                character.velocity += impulse;
                character.floor = None;
            }
        }

        // Get the raw 2D input vector
        let input_vec = q_input.action::<Move>().value().as_axis2d();

        // Rotate the movement direction vector by the camera's yaw
        let mut move_input =
            (transform.rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y)).normalize_or_zero();

        let max_acceleration = match character.floor {
            Some(floor_normal) => {
                apply_friction(&mut character.velocity, EXAMPLE_FRICTION, time.delta_secs());

                // Make sure velocity is never towards the floor since this makes the jump height inconsistent
                let downward_vel = character.velocity.dot(*floor_normal).min(0.0);
                character.velocity -= floor_normal * downward_vel;

                EXAMPLE_FLOOR_ACCELERATION
            }
            None => {
                // Apply gravity when not grounded
                let gravity = character.up * -EXAMPLE_GRAVITY * time.delta_secs();
                character.velocity += gravity;

                EXAMPLE_AIR_ACCELERATION
            }
        };

        // Filter out the character entity as well as any entities not in the character's collision filter
        let mut filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity])
            .with_mask(layers.filters);

        // Also filter out sensor entities
        filter.excluded_entities.extend(sensors);

        let up = character.up;

        // Check if the floor is walkable
        let is_walkable = |normal| {
            let slope_angle = up.angle_between(normal);
            slope_angle < EXAMPLE_WALKABLE_ANGLE
        };

        let config = MoveAndSlideConfig::default();

        // Sweep in input direction to determine how to project the input direction.
        if let Some((motion, hit)) = character_sweep(
            collider,
            config.epsilon,
            transform.translation,
            move_input * max_acceleration.min(EXAMPLE_MOVEMENT_SPEED) * time.delta_secs(),
            transform.rotation,
            &spatial_query,
            &filter,
        ) {
            // Move to the wall or slope
            transform.translation += motion;

            match is_walkable(hit.normal1) {
                // When on a walkable surface (floor or gentle slope):
                // Project the movement vector to follow the slope while maintaining horizontal intent
                true => {
                    move_input = project_vector_on_floor(
                        move_input,
                        Dir3::new(hit.normal1).unwrap(),
                        character.up,
                    );
                }
                // When encountering a non-walkable surface (wall or steep slope):
                // Project the movement vector to slide along the wall and prevent climbing
                false => {
                    move_input = project_vector_on_wall(
                        move_input,
                        Dir3::new(hit.normal1).unwrap(),
                        character.up,
                    );
                }
            }
        }

        // accelerate in the movement direction
        if let Ok((direction, throttle)) = Dir3::new_and_length(move_input) {
            accelerate(
                &mut character.velocity,
                direction,
                max_acceleration * throttle,
                EXAMPLE_MOVEMENT_SPEED,
                time.delta_secs(),
            );
        }

        let rotation = transform.rotation;

        let mut floor = None;

        move_and_slide(
            &spatial_query,
            collider,
            &mut transform.translation,
            &mut character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |hit| {
                if is_walkable(hit.normal1) {
                    floor = Some(Dir3::new(hit.normal1).unwrap());
                }
            },
        );

        // Check for floor when previously on the floor and no floor was found during move and slide
        // to avoid rapid changes to the grounded state
        if character.floor.is_some() && floor.is_none() {
            if let Some((movement, hit)) = character_sweep(
                collider,
                config.epsilon,
                transform.translation,
                -character.up * 10.0, // arbitrary trace distance
                rotation,
                &spatial_query,
                &filter,
            ) {
                if is_walkable(hit.normal1) {
                    transform.translation += movement; // also snap to the floor
                    floor = Some(Dir3::new(hit.normal1).unwrap());
                }
            }
        }

        character.floor = floor;
    }
}

/// Projects a movement vector onto a walkable floor or slope surface
pub fn project_vector_on_floor(vector: Vec3, floor_normal: Dir3, up: Dir3) -> Vec3 {
    // Split input vector into vertical and horizontal components
    let mut vertical = vector.project_onto(*up);
    let mut horizontal = vector - vertical;

    // Remove downward velocity
    if vertical.dot(*up) < 0.0 {
        vertical = Vec3::ZERO;
    }

    let Ok(tangent) = Dir3::new(horizontal.cross(*up)) else {
        return vertical; // No horizontal velocity
    };

    // Calculate the horizontal direction along the slope
    // This gives us a vector that follows the slope but maintains original horizontal intent
    let Ok(horizontal_direction) = Dir3::new(tangent.cross(*floor_normal)) else {
        return vertical + horizontal; // Horizontal direction is perpendicular with the floor normal
    };

    // Project horizontal movement onto the calculated direction
    horizontal = horizontal.project_onto_normalized(*horizontal_direction);

    vertical + horizontal
}

/// Projects a movement vector against a wall or non-walkable slope
pub fn project_vector_on_wall(vector: Vec3, floor_normal: Dir3, up: Dir3) -> Vec3 {
    // Split input vector into vertical and horizontal components
    let mut vertical = vector.project_onto(*up);
    let mut horizontal = vector - vertical;

    vertical = vertical.reject_from_normalized(*floor_normal);

    let Ok(tangent) = Dir3::new(floor_normal.cross(*up)) else {
        return vertical + horizontal; // This is not a wall
    };

    // Project horizontal movement along the wall tangent
    horizontal = horizontal.project_onto_normalized(*tangent);

    vertical + horizontal
}

/// This is a simple example inspired by Quake, users are expected to bring their own logic for acceleration.
fn accelerate(
    velocity: &mut Vec3,
    direction: Dir3,
    max_acceleration: f32,
    target_speed: f32,
    delta: f32,
) {
    // Current speed in the desired direction.
    let current_speed = velocity.dot(*direction);

    // No acceleration is needed if current speed exceeds target.
    if current_speed >= target_speed {
        return;
    }

    // Clamp to avoid acceleration past the target speed.
    let accel_speed = f32::min(target_speed - current_speed, max_acceleration * delta);

    *velocity += accel_speed * direction;
}

/// Constant acceleration in the opposite direction of velocity.
pub fn apply_friction(velocity: &mut Vec3, friction: f32, delta: f32) {
    let speed_sq = velocity.length_squared();

    if speed_sq < 1e-4 {
        return;
    }

    let factor = f32::exp(-friction / speed_sq.sqrt() * delta);

    *velocity *= factor;
}
