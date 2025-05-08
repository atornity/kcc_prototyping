use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    camera::MainCamera, character::*, input::{self, DefaultContext, Jump}, move_and_slide::*
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
    ground: Option<Dir3>,
    up: Dir3,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            ground: None,
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
    for (entity, actions, mut transform, mut character, character_collider, layers) in &mut q_kcc {
        if actions.action::<Jump>().state() == ActionState::Fired {
            if character.ground.is_some() {
                let impulse = Vec3::Y * EXAMPLE_JUMP_IMPULSE;
                character.velocity += impulse;
                character.ground = None;
            }
        }

        // Get the raw 2D input vector
        let input_vec = actions.action::<input::Move>().value().as_axis2d();

        // Rotate the movement direction vector by the camera's yaw
        let mut direction =
            main_camera_transform.rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y);

        let max_acceleration = match character.ground {
            Some(floor_normal) => {
                let friction = friction(character.velocity, EXAMPLE_FRICTION, time.delta_secs());
                character.velocity += friction;

                // Make sure velocity is never towards the floor since this makes the jump height inconsistent
                let downward_vel = character.velocity.dot(*floor_normal).min(0.0);
                character.velocity -= floor_normal * downward_vel;

                // Project input direction on the floor normal to allow walking down slopes
                // TODO: this is wrong, walking diagonally up/down slopes will be slightly off direction wise,
                // even more so for steep slopes.
                direction = direction
                    .reject_from_normalized(*floor_normal)
                    .normalize_or_zero();

                EXAMPLE_GROUND_ACCELERATION
            }
            None => {
                // Apply gravity when not grounded
                let gravity = Vec3::Y * -EXAMPLE_GRAVITY * time.delta_secs();
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
        
        let mut grounded_this_frame = false;

        if let Some(move_and_slide_result) = move_and_slide(
            &spatial_query,
            &character_collider,
            transform.translation,
            character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |hit| {
                let walkable = is_walkable(hit.shape_hit, Dir3::Y, EXAMPLE_WALKABLE_ANGLE);

                if walkable {
                    grounded_this_frame = true;
                    character.ground = Some(Dir3::new(hit.shape_hit.normal1).unwrap_or(Dir3::Y));
                }

                // In order to try step up we need to be grounded and hitting a "wall".
                if !walkable && character.ground.is_some() {

                    // See notes below about why we use a cylinder collider in the next two stages.
                    let cylinder_collider = Collider::cylinder(
                        EXAMPLE_CHARACTER_RADIUS,
                        // need to add both radii to length to account for the capsule's hemispheres on both ends
                        // because the goal is to have the cylinder collider be the same height as the capsule collider
                        EXAMPLE_CHARACTER_CAPSULE_LENGTH + (2.0 * EXAMPLE_CHARACTER_RADIUS)
                    );

                    if let Some(try_climb_step_result) = try_climb_step(
                        // We use a cylinder collider for climbing up steps
                        // because it helps climb them even at low speeds.
                        &cylinder_collider,
                        &config,
                        *hit.overridable_translation,
                        hit.remaining_motion,
                        rotation,
                        &spatial_query,
                        EXAMPLE_STEP_HEIGHT,
                        EXAMPLE_GROUND_CHECK_DISTANCE,
                        &filter,
                    ) {
                        // Since we're a capsule, we need to zero out the Y velocity when 
                        // we climb a step or it will feel like we're launching over the step.
                        // This is because of the roundness of the capsule naturally pushing 
                        // us up as we encroach on the step, especially at high speeds.
                        hit.overridable_velocity.y = 0.0;

                        // We need to override the translation here because the we stepped up
                        *hit.overridable_translation = try_climb_step_result.new_translation;

                        // Check the ground again after climbing the step
                        // because it's possible what we stepped up onto is 
                        // not walkable for whatever reason. Also a good failsafe.
                        // note:    Maybe the gamedev can have something 
                        //          that is a non-wall but also not walkable?
                        //          In which case, this would be important.
                        if let Some((_movement, hit_normal)) = ground_check(
                            // We use a cylinder collider for this specific ground check
                            // because I find it gives a better hit normal to slide along.
                            &cylinder_collider,
                            &config,
                            *hit.overridable_translation,
                            Dir3::Y,
                            rotation,
                            &spatial_query,
                            &filter,
                            EXAMPLE_GROUND_CHECK_DISTANCE,
                            EXAMPLE_WALKABLE_ANGLE,
                        ) {
                            // Finally, we need to override the normal so that when we slide, 
                            // we slide along the normal of the new floor and not the wall of the step.
                            *hit.overridable_normal = hit_normal;
                            grounded_this_frame = true;
                            character.ground = Some(Dir3::new(hit_normal).unwrap_or(Dir3::Y));
                        }
                        else {
                            character.ground = None;
                        }
                    }
                }
            },
        ) {
            transform.translation = move_and_slide_result.new_translation;
            character.velocity = move_and_slide_result.new_velocity;
        }

        if !grounded_this_frame {
            if let Some((movement, hit_normal)) = ground_check(
                &character_collider,
                &config,
                transform.translation,
                Dir3::Y,
                rotation,
                &spatial_query,
                &filter,
                EXAMPLE_GROUND_CHECK_DISTANCE,
                EXAMPLE_WALKABLE_ANGLE,
            ) {
                transform.translation -= movement * Vec3::Y;
                character.ground = Some(Dir3::new(hit_normal).unwrap_or(Dir3::Y));
            }
            else {
                character.ground = None;
            }
        }
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
