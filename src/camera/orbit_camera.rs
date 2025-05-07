use super::{Attachments, ViewAngles};
use crate::{
    AttachedTo,
    input::{OrbitCameraContext, OrbitZoom},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_origin.after(super::view_input),
            zoom_input,
            update_spring_arm,
        )
            .chain(),
    );
}

#[derive(Component, Reflect, Default, Debug, PartialEq, Clone, Copy)]
#[reflect(Component)]
#[require(FollowOffset)]
pub(crate) struct FollowOrigin(pub Vec3);

/// By default the center of orbit will be equal to the position of the target.
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct FollowOffset {
    pub absolute: Vec3,
    pub relative: Vec3,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub(crate) struct SpringArm {
    pub distance: f32,
    pub target_distance: f32,
    pub recover_speed: f32,
    pub collision_radius: f32,
    pub filters: LayerMask,
}

impl Default for SpringArm {
    fn default() -> Self {
        Self {
            distance: 4.0,
            target_distance: 4.0,
            recover_speed: 6.0,
            collision_radius: 0.2,
            filters: LayerMask::ALL,
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub(crate) struct FirstPersonCamera;

pub(crate) fn zoom_input(
    targets: Query<(&Actions<OrbitCameraContext>, &Attachments)>,
    mut cameras: Query<&mut SpringArm>,
) -> Result {
    for (actions, owned_cameras) in &targets {
        let mut iter = cameras.iter_many_mut(owned_cameras.iter());
        while let Some(mut arm) = iter.fetch_next() {
            let zoom_input = actions.action::<OrbitZoom>().value().as_axis2d();
            let zoom_delta = zoom_input.y * arm.distance * 0.1; // TODO: configurable speed

            arm.distance -= zoom_delta;
            arm.distance = arm.distance.clamp(0.1, 100.0); // TODO: configurable range
        }
    }

    Ok(())
}

pub(crate) fn update_origin(
    targets: Query<&GlobalTransform>,
    mut cameras: Query<(
        &mut FollowOrigin,
        &mut Transform,
        &ViewAngles,
        &FollowOffset,
        &AttachedTo,
    )>,
) -> Result {
    for (mut origin, mut transform, angles, offset, attached_to) in &mut cameras {
        let orbit_transform = targets.get(attached_to.0)?;

        let mut point = orbit_transform.translation();

        point += offset.absolute;
        point += angles.to_quat() * offset.relative;

        origin.0 = point;
        transform.translation = point;
    }

    Ok(())
}

pub(crate) fn update_spring_arm(
    spatial_query: SpatialQuery,
    mut cameras: Query<(
        &mut SpringArm,
        &mut Transform,
        &FollowOrigin,
        &AttachedTo,
        Has<FirstPersonCamera>,
    )>,
    time: Res<Time>,
) {
    for (mut arm, mut camera_transform, origin, attached_to, first_person) in &mut cameras {
        let direction = camera_transform.rotation * Dir3::Z;

        let filter =
            SpatialQueryFilter::from_mask(arm.filters).with_excluded_entities([attached_to.0]);

        // Smoothly interpolate to an arm distance of 0.0 when in first person mode
        if first_person {
            arm.distance = arm
                .distance
                .lerp(0.0, arm.recover_speed * time.delta_secs());
        } else if let Some(hit) = spatial_query.cast_shape(
            &Collider::sphere(arm.collision_radius),
            origin.0,
            Quat::IDENTITY,
            direction,
            &ShapeCastConfig {
                max_distance: arm.target_distance,
                ..Default::default()
            },
            &filter,
        ) {
            // If there's a collision, quickly snap to the hit distance to avoid clipping with the world
            arm.distance = hit.distance;
        } else {
            // Otherwise, interpolate to the target distance
            let distance = arm
                .distance
                .lerp(arm.target_distance, arm.recover_speed * time.delta_secs());
            arm.distance = distance;
        }

        camera_transform.translation = origin.0 + direction * arm.distance;
    }
}
