use bevy::prelude::*;
use avian3d::prelude::*;

use crate::move_and_slide::*;

pub const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
pub const EXAMPLE_CHARACTER_CAPSULE_LENGTH: f32 = 1.0; 
pub const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
pub const EXAMPLE_GROUND_ACCELERATION: f32 = 100.0;
pub const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
pub const EXAMPLE_FRICTION: f32 = 60.0;
pub const EXAMPLE_WALKABLE_ANGLE: f32 = std::f32::consts::PI / 4.0;
pub const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
pub const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
pub const EXAMPLE_STEP_HEIGHT: f32 = 0.25;
pub const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.1;

// @todo: probably want to improve the ergonomics of these 
// functions by accepting a struct instead of a bunch of arguments, 
// that way each can be commented and we can also provide sane defaults, and ordering doesn't matter.

/// Checks if a surface is walkable based on its slope angle and the up direction.
pub fn is_walkable(hit: ShapeHitData, up: Dir3, walkable_angle: f32) -> bool {
    let slope_angle = up.angle_between(hit.normal1);
    slope_angle < walkable_angle
}

/// Result of trying to climb a step.
pub struct TryClimbStepResult {
    pub new_translation: Vec3,
    pub new_normal: Vec3,
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
    collider: &Collider,
    config: &MoveAndSlideConfig,
    translation: Vec3,
    motion: Vec3,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    step_up_height: f32,
    ground_check_distance: f32,
    filter: &SpatialQueryFilter,
) -> Option<TryClimbStepResult> {
    
    let step_up_pos = translation + Vec3::Y * (step_up_height + ground_check_distance);

    let step_up_hit = sweep_check(
        collider,
        config.epsilon,
        step_up_pos,
        Dir3::new(motion).unwrap_or(Dir3::Z),
        motion.length(),
        rotation,
        spatial_query,
        filter,
    );
    
    // We hit something at the step height, so we can't climb it.
    if step_up_hit.is_some() {
        return None;
    }

    let step_down_pos = step_up_pos + motion;
    let step_down_hit = sweep_check(
        collider,
        config.epsilon,
        step_down_pos,
        Dir3::NEG_Y,
        step_up_height + ground_check_distance,
        rotation,
        spatial_query,
        filter,
    );

    if let Some((hit_distance, hit)) = step_down_hit {        
        Some(TryClimbStepResult {
            new_translation: step_up_pos + motion + Dir3::NEG_Y * hit_distance,
            new_normal: hit.normal1,
        })
    } else {
        None
    }
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
    ground_check_distance: f32,
    walkable_angle: f32,
) -> Option<(f32, Vec3)> {
    if let Some((movement, hit)) = sweep_check(
        collider,
        config.epsilon,
        translation,
        -up,
        ground_check_distance,
        rotation,
        spatial_query,
        filter,
    ) {
        if is_walkable(hit, up, walkable_angle) {
            Some((movement, hit.normal1))
        } else {
            None
        }
    } else {
        None
    }
}
