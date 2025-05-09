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
pub const EXAMPLE_JUMP_IMPULSE: f32 = 8.0;
pub const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
pub const EXAMPLE_STEP_HEIGHT: f32 = 0.25;
pub const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.1;

// @todo: probably want to improve the ergonomics of these
// functions by accepting a struct instead of a bunch of arguments,
// that way each can be commented and we can also provide sane defaults, and ordering doesn't matter.

/// Represents the ground a character is currently standing on.
#[derive(Reflect, Debug, Clone, Copy)]
pub struct Ground {
    pub entity: Entity,
    pub normal: Dir3,
    pub distance: f32,
}

impl Ground {
    /// Construct a new [`Ground`] if the `normal` is walkable with the given `walkable_angle` and `up` direction.
    ///
    /// Returns `None` if the [`Ground`] isn't walkable or if the provided `normal` can't be normalized.
    pub fn new_if_walkable(
        entity: Entity,
        normal: impl TryInto<Dir3>,
        distance: f32,
        up: Dir3,
        walkable_angle: f32,
    ) -> Option<Self> {
        let normal = normal.try_into().ok()?;

        if normal.angle_between(*up) > walkable_angle {
            return None;
        }

        Some(Self {
            entity,
            normal,
            distance,
        })
    }

    /// Returns `true` if the [`Ground`] is walkable with the given `walkable_angle` and `up` direction.
    pub fn is_walkable(&self, up: Dir3, walkable_angle: f32) -> bool {
        self.normal.angle_between(*up) <= walkable_angle
    }
}

/// Perform a [`sweep_check`] in the opposing direction of `up` and returns the [`Ground`] if it's walkable.
pub fn ground_check(
    collider: &Collider,
    origin: Vec3,
    rotation: Quat,
    up: Dir3,
    max_distance: f32,
    epsilon: f32,
    walkable_angle: f32,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> Option<Ground> {
    let (safe_distance, hit) = sweep_check(
        collider,
        origin,
        rotation,
        -up,
        max_distance,
        epsilon,
        spatial_query,
        filter,
    )?;

    Ground::new_if_walkable(hit.entity, hit.normal1, safe_distance, up, walkable_angle)
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
    translation: Vec3,
    motion: Vec3,
    rotation: Quat,
    up: Dir3,
    step_up_height: f32,
    epsilon: f32,
    spatial_query: &SpatialQuery,
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
        step_down_pos,
        rotation,
        -up,
        step_up_height,
        epsilon,
        spatial_query,
        filter,
    )?;

    let new_translation = step_down_pos - up * safe_distance;

    Some((new_translation, step_down_hit))
}
