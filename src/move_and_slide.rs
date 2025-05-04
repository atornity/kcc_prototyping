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
    let original_velocity = *velocity;

    let mut remaining_motion = *velocity * delta_time;

    for _ in 0..config.max_iterations {
        let Some((safe_movement, hit)) = character_sweep(
            collider,
            config.epsilon,
            *translation,
            remaining_motion,
            rotation,
            spatial_query,
            filter,
        ) else {
            // No collision, move the full remaining distance
            *translation += remaining_motion;
            break;
        };

        // Move the transform to just before the point of collision
        *translation += safe_movement;

        // Update the velocity by how much we moved
        remaining_motion -= safe_movement;

        // Project velocity and remaining motion onto the surface plane
        remaining_motion = remaining_motion.reject_from(hit.normal1);
        *velocity = velocity.reject_from(hit.normal1);

        // Trigger callbacks
        on_hit(hit);

        // Quake2: "If velocity is against original velocity, stop early to avoid tiny oscilations in sloping corners."
        if remaining_motion.dot(original_velocity) <= 0.0 {
            break;
        }
    }
}

fn similar_plane(a: Vec3, b: Vec3) -> bool {
    // Check if the two vectors are similar by comparing their dot product with a threshold
    a.dot(b) > (1.0 - f32::EPSILON)
}

fn solve_collision_planes(
    velocity: Vec3,
    hits: &[Vec3],
    original_velocity_direction: Vec3,
) -> Vec3 {
    // Early out if we have no velocity or no hits
    if velocity.length_squared() <= 0.0 || original_velocity_direction.length_squared() <= 0.0 {
        return Vec3::ZERO;
    }

    if hits.is_empty() {
        return velocity;
    }

    // Do our initial rejection to calculate the sliding velocity.
    let first_hit_normal = hits[hits.len() - 1];
    let velocity_direction = velocity.normalize_or_zero();
    if velocity_direction.dot(first_hit_normal) >= 0.0 {
        return velocity;
    }
    let initial_velocity = velocity.reject_from(first_hit_normal);

    // Join the original velocity direction as an additional constraining plane
    let original_velocity_normal = original_velocity_direction.normalize_or_zero();
    let all_hits: Vec<Vec3> = std::iter::once(original_velocity_normal)
        .chain(hits.iter().cloned())
        .collect();

    // We should filter out any normals that are similar to the existing constraints
    let mut filtered_hits = all_hits.iter().filter(|&n| {
        !similar_plane(first_hit_normal, *n) && !similar_plane(original_velocity_normal, *n)
    });

    filtered_hits.try_fold(initial_velocity, |vel, second_hit_normal| {
        let vel = vel.reject_from(*second_hit_normal);
        let vel_dir = vel.normalize_or_zero();

        // If the velocity is already parallel to the first hit normal, we can return it directly
        if similar_plane(vel_dir, first_hit_normal) {
            // If the velocity is small enough we can just assume we have no reason to move
            if vel.length_squared() <= f32::EPSILON {
                Err(vel)
                // Otherwise we need to keep working.
            } else {
                Ok(vel)
            }
        } else {
            // If we have a valid second hit normal, we can calculate the crease direction
            let crease_dir = first_hit_normal.cross(*second_hit_normal).normalize_or_zero();
            let vel_proj = vel.project_onto(crease_dir);
            let vel_proj_dir = vel_proj.normalize_or_zero();

            // Check if the velocity projection is a corner case
            // A corner case is when the velocity projection is not similar to either of the hit normals
            // but is similar to the crease direction formed by the two hit normals.
            let is_corner = all_hits.iter().any(|third_hit_normal| {
                !similar_plane(first_hit_normal, *third_hit_normal) &&
                !similar_plane(*second_hit_normal, *third_hit_normal) &&
                similar_plane(vel_proj_dir, *third_hit_normal)
            });

            // If we are in a corner case we return a zero vector
            if is_corner {
                Err(Vec3::ZERO)
                // Otherwise we can return the velocity if we have a small enough projection
            } else if vel_proj.length_squared() <= f32::EPSILON {
                Err(vel_proj)
            } else {
                // Otherwise lets keep working with the projection
                Ok(vel_proj)
            }
        }
    }).unwrap_or_else(|vel| vel)
}