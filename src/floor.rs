use avian3d::prelude::*;
use bevy::prelude::*;

use crate::move_and_slide;

#[derive(Reflect, Debug, Clone, Copy)]
pub(crate) struct Floor {
    pub entity: Entity,
    pub normal: Dir3,
    pub distance: f32,
}

impl Floor {
    /// Construct a new [`Floor`] if the `normal` is walkable with the given `walkable_angle` and `up` direction.
    pub fn new_if_walkable<D: TryInto<Dir3>>(
        entity: Entity,
        normal: D,
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
}

/// Sweep down against the `up` direction, returns the [`Floor`] if there's a walkable hit.
pub(crate) fn find_floor(
    spatial_query: &SpatialQuery,
    collider: &Collider,
    origin: Vec3,
    rotation: Quat,
    up: Dir3,
    max_distance: f32,
    epsilon: f32,
    walkable_angle: f32,
    filter: &SpatialQueryFilter,
) -> Option<Floor> {
    let (safe_distance, hit) = move_and_slide::character_sweep(
        spatial_query,
        collider,
        origin,
        rotation,
        -up,
        max_distance,
        epsilon,
        filter,
    )?;

    Floor::new_if_walkable(hit.entity, hit.normal1, safe_distance, up, walkable_angle)
}
