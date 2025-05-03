use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, ShapeCastConfig, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    DefaultCamera, KCCMarker,
    input::{DefaultContext, Jump, Move},
};

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, movement);
    }
}

#[derive(Component)]
pub struct KinematicVelocity(pub Vec3);

#[derive(Bundle)]
pub struct KCCBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub kcc_marker: KCCMarker,
    pub kinematic_velocity: KinematicVelocity,
}

impl Default for KCCBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(0.35, 1.0),
            rigid_body: RigidBody::Kinematic,
            kcc_marker: KCCMarker,
            kinematic_velocity: KinematicVelocity(Vec3::ZERO),
        }
    }
}

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;

fn movement(
    mut q_kcc: Query<
        (
            Entity,
            &mut Transform,
            &mut KinematicVelocity,
            &Collider,
            &CollisionLayers,
        ),
        With<KCCMarker>,
    >,
    q_input: Single<&Actions<DefaultContext>>,
    q_camera: Query<&Transform, (With<DefaultCamera>, Without<KCCMarker>)>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    // get camera rotation yaw
    let Some(camera_transform) = q_camera.single().ok() else {
        warn!("No camera found!");
        return;
    };

    if q_input.action::<Jump>().state() == ActionState::Fired {
        println!("Jump action fired!");
    }

    // Get the raw 2D input vector
    let input_vec = q_input.action::<Move>().value().as_axis2d();

    let Some((entity, mut kcc_transform, mut kinematic_vel, collider, layers)) =
        q_kcc.single_mut().ok()
    else {
        warn!("No KCC found!");
        return;
    };

    // Rotate the movement direction vector by the camera's yaw
    // movement_dir = Quat::from_rotation_y(camera_yaw) * movement_dir;
    let direction = kcc_transform
        .rotation
        .mul_vec3(Vec3::new(input_vec.x, 0.0, -input_vec.y))
        .normalize_or_zero();

    let mut artifical_velocity = KinematicVelocity(direction * EXAMPLE_MOVEMENT_SPEED);

    // Apply the final movement
    // kcc_transform.translation += direction * EXAMPLE_MOVEMENT_SPEED * time.delta_secs();

    let filter = SpatialQueryFilter::from_mask(layers.filters).with_excluded_entities([entity]);

    move_and_slide(
        MoveAndSlideConfig::default(),
        collider,
        time.delta_secs(),
        &mut kcc_transform,
        &mut artifical_velocity,
        &spatial_query,
        &filter,
    )
}

////// EXAMPLE MOVEMENT /////////////
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

pub fn move_and_slide(
    config: MoveAndSlideConfig,
    collider: &Collider,
    delta_time: f32,
    transform: &mut Transform,
    velocity: &mut KinematicVelocity,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) {
    let mut remaining_velocity = velocity.0 * delta_time;
    for _ in 0..config.max_iterations {
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            transform.translation,
            transform.rotation,
            Dir3::new(remaining_velocity.normalize_or_zero()).unwrap_or(Dir3::X),
            &ShapeCastConfig::from_max_distance(remaining_velocity.length()),
            &filter,
        ) {
            // Calculate our safe distances to move
            let safe_distance = (hit.distance - config.epsilon).max(0.0);

            // How far is safe to translate by
            let safe_movement = remaining_velocity * safe_distance;

            // Move the transform to just before the point of collision
            transform.translation += safe_movement;

            // Update the velocity by how much we moved
            remaining_velocity -= safe_movement;

            // Project velocity onto the surface plane
            remaining_velocity = remaining_velocity.reject_from(hit.normal1);

            if remaining_velocity.dot(velocity.0) < 0.0 {
                // Don't allow sliding back into the surface
                remaining_velocity = Vec3::ZERO;
                break;
            }
        } else {
            // No collision, move the full remaining distance
            transform.translation += remaining_velocity;
            break;
        }
    }

    // Update the velocity for the next frame
    velocity.0 = remaining_velocity;
}
