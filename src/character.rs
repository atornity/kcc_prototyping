use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::move_and_slide::*;

pub const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
pub const EXAMPLE_CHARACTER_CAPSULE_LENGTH: f32 = 1.0;
pub const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
pub const EXAMPLE_GROUND_ACCELERATION: f32 = 100.0;
pub const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
pub const EXAMPLE_FRICTION: f32 = 60.0;
pub const EXAMPLE_WALKABLE_ANGLE: f32 = PI / 4.0;
pub const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
pub const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
pub const EXAMPLE_STEP_HEIGHT: f32 = 0.25;
pub const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.1;

// @todo: probably want to improve the ergonomics of these
// functions by accepting a struct instead of a bunch of arguments,
// that way each can be commented and we can also provide sane defaults, and ordering doesn't matter.

/// Checks if a surface is walkable based on its slope angle and the up direction.
pub fn is_walkable(normal: Vec3, up: Dir3, walkable_angle: f32) -> bool {
    let slope_angle = up.angle_between(normal);
    slope_angle < walkable_angle
}

/// Find and climb steps in the movement direction.
///
/// # Prerequisites
/// Before calling this function, it is recommended that:
/// - The character is grounded
/// - The character is hitting a wall
///
/// # How This Works
///
/// The step climbing process follows these steps:
///
/// 1. **Step Height Check**
///    - Perform a shape-cast from the character's position
///    - Cast is done above ground level by the step height
///    - Cast is in the direction of movement
///    - If nothing is hit, the wall is "step-uppable"
///
/// 2. **Height Discovery**
///    - If the above shape-cast hits nothing:
///      - Perform a downward shape-cast from the step up height
///        at the intended step up position based on the remaining motion.
///     - This determines the actual height of the step
///
/// 3. **Step Execution**
///    - Teleport up by the discovered height
///    - Move forward with remaining motion
pub fn try_climb_step(
    spatial_query: &SpatialQuery,
    collider: &Collider,
    translation: Vec3,
    motion: Vec3,
    rotation: Quat,
    up: Dir3,
    step_up_height: f32,
    epsilon: f32,
    filter: &SpatialQueryFilter,
) -> Option<(Vec3, ShapeHitData)> {
    let step_up_pos = translation + up * step_up_height;

    let horizontal_motion = motion.reject_from_normalized(Vec3::Y);

    // Only step up if horizontal motion is non zero
    if let Ok(direction) = Dir3::new(horizontal_motion) {
        // Step up and sweep forward
        let None = spatial_query.cast_shape(
            collider,
            step_up_pos,
            rotation,
            direction,
            &ShapeCastConfig {
                max_distance: horizontal_motion.length(),
                ..Default::default()
            },
            filter,
        ) else {
            // We hit something at the step height, so we can't climb it.
            return None;
        };
    }

    let step_down_pos = step_up_pos + horizontal_motion;

    // Step forward and sweep down
    let (safe_distance, step_down_hit) = sweep_check(
        collider,
        epsilon,
        step_down_pos,
        -up,
        step_up_height,
        rotation,
        spatial_query,
        filter,
    )?;

    let new_translation = step_down_pos - up * safe_distance;

    Some((new_translation, step_down_hit))
}

/// Check if the character is grounded and update ground state accordingly
/// Applies a downward sweep to check for a valid ground.
/// Returns the new translation after snapping to the ground
pub fn ground_check(
    collider: &Collider,
    config: &MoveAndSlideConfig,
    translation: Vec3,
    up: Dir3,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
    floor_check_distance: f32,
    walkable_angle: f32,
) -> Option<(f32, ShapeHitData)> {
    let (safe_distance, hit) = sweep_check(
        collider,
        config.epsilon,
        translation,
        -up,
        floor_check_distance,
        rotation,
        spatial_query,
        filter,
    )?;

    if !is_walkable(hit.normal1, up, walkable_angle) {
        return None;
    }

    Some((safe_distance, hit))
}
