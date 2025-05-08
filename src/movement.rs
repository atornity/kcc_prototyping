use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    camera::MainCamera,
    input::{self, DefaultContext, Jump},
    move_and_slide::{Floor, MoveAndSlideConfig, floor_sweep, move_and_slide},
};

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
    floor: Option<Floor>,
    up: Dir3,
}

impl Character {
    pub fn launch(&mut self, impulse: Vec3) {
        if let Some(floor) = self.floor {
            if floor.normal.dot(impulse) > 0.0 {
                self.floor = None;
            }
        }

        self.velocity += impulse
    }

    pub fn jump(&mut self, impulse: f32) {
        let down = self.velocity.dot(*self.up).min(0.0);
        // Override downward velocity only to allow jumping when walking up ramps
        let vertical = self.up * -down;
        self.launch(self.up * impulse + vertical);
    }

    pub fn grounded(&self) -> bool {
        self.floor.is_some()
    }
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

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
const EXAMPLE_FLOOR_ACCELERATION: f32 = 100.0;
const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
const EXAMPLE_FRICTION: f32 = 60.0;
const EXAMPLE_WALKABLE_ANGLE: f32 = PI / 4.0;
const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games

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
        }

        // Get the raw 2D input vector
        let input_vec = actions.action::<input::Move>().value().as_axis2d();

        // Rotate the movement direction vector by the camera's yaw
        let mut direction =
            main_camera_transform.rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y);

        let max_acceleration = match character.floor {
            Some(floor) => {
                let friction = friction(character.velocity, EXAMPLE_FRICTION, time.delta_secs());
                character.velocity += friction;

                // Make sure velocity is never towards the floor since this makes the jump height inconsistent
                let downward_vel = character.velocity.dot(*floor.normal).min(0.0);
                character.velocity -= floor.normal * downward_vel;

                // Project input direction on the floor normal to allow walking down slopes
                // TODO: this is wrong, walking diagonally up/down slopes will be slightly off direction wise,
                // even more so for steep slopes.
                direction = direction
                    .reject_from_normalized(*floor.normal)
                    .normalize_or_zero();

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

        let up = character.up;

        // Check if the floor is walkable
        let is_walkable = |normal: Vec3| {
            let slope_angle = up.angle_between(normal);
            slope_angle < EXAMPLE_WALKABLE_ANGLE
        };

        let mut floor = None;

        if let Some(move_and_slide_result) = move_and_slide(
            &spatial_query,
            collider,
            transform.translation,
            character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |hit| {
                if is_walkable(hit.normal1) {
                    floor = Some(Floor {
                        entity,
                        normal: Dir3::new(hit.normal1).unwrap(),
                        distance: hit.distance, // TODO: use safe distance?
                    });
                }
            },
        ) {
            transform.translation = move_and_slide_result.new_translation;
            character.velocity = move_and_slide_result.new_velocity;
        }

        // Check for floor when previously on the floor and no floor was found during move and slide
        // to avoid rapid changes to the grounded state
        if character.floor.is_some() && floor.is_none() {
            if let Some(flr) = floor_sweep(
                collider,
                config.epsilon,
                transform.translation,
                -character.up,
                0.1,
                rotation,
                &spatial_query,
                &filter,
            ) {
                if is_walkable(*flr.normal) {
                    transform.translation -= character.up * flr.distance; // also snap to the floor
                    floor = Some(flr);
                }
            }
        }

        character.floor = floor;
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
