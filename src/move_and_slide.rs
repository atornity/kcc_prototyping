use avian3d::prelude::*;
use bevy::prelude::*;

#[must_use]
pub fn character_sweep(
    collider: &Collider,
    epsilon: f32,
    origin: Vec3,
    motion: Vec3,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> Option<(Vec3, ShapeHitData)> {
    let (direction, length) = Dir3::new_and_length(motion).ok()?;

    let hit = spatial_query.cast_shape(
        collider,
        origin,
        rotation,
        direction,
        &ShapeCastConfig {
            max_distance: length + epsilon, // extend the trace slightly
            target_distance: epsilon, // I'm not sure what this does but I think this is correct ;)
            ignore_origin_penetration: true,
            ..Default::default()
        },
        filter,
    )?;

    // How far is safe to translate by
    let safe_movement = direction * (hit.distance - epsilon).max(0.0);

    Some((safe_movement, hit))
}

////// EXAMPLE MOVEMENT /////////////
#[derive(Clone, Copy)]
pub struct MoveAndSlideConfig {
    pub max_iterations: usize,
    pub epsilon: f32,
}

impl Default for MoveAndSlideConfig {
    fn default() -> Self {
        Self {
            max_iterations: 4,
            epsilon: 0.01,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn move_and_slide(
    spatial_query: &SpatialQuery,
    collider: &Collider,
    translation: &mut Vec3,
    velocity: &mut Vec3,
    rotation: Quat,
    config: MoveAndSlideConfig,
    filter: &SpatialQueryFilter,
    delta_time: f32,
    mut on_hit: impl FnMut(ShapeHitData),
) {
    let Ok(original_direction) = Dir3::new(*velocity) else {
        return;
    };

    for _ in 0..config.max_iterations {
        let motion = *velocity * delta_time;

        let Some((safe_movement, hit)) = character_sweep(
            collider,
            config.epsilon,
            *translation,
            motion,
            rotation,
            spatial_query,
            filter,
        ) else {
            // No collision, move the full remaining distance
            *translation += motion;
            break;
        };

        // Trigger callbacks
        on_hit(hit);

        // Move the transform to just before the point of collision
        *translation += safe_movement;

        // Project velocity and remaining motion onto the surface plane
        *velocity = velocity.reject_from(hit.normal1);

        // Quake2: "If velocity is against original velocity, stop early to avoid tiny oscilations in sloping corners."
        if velocity.dot(*original_direction) <= 0.0 {
            break;
        }
    }
}
