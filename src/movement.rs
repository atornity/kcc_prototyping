use std::f32::consts::PI;

use avian3d::prelude::{Collider, RigidBody, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};
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
    mut q_kcc: Query<(Entity, &mut Transform, &mut KinematicVelocity, &Collider), With<KCCMarker>>,
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

    let Some((entity, mut kcc_transform, mut kinematic_vel, collider)) = q_kcc.single_mut().ok()
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

    move_and_slide(
        MoveAndSlideConfig::default(),
        collider,
        time.delta_secs(),
        &entity,
        &mut kcc_transform,
        &mut artifical_velocity,
        &spatial_query,
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
    entity: &Entity,
    transform: &mut Transform,
    velocity: &mut KinematicVelocity,
    spatial_query: &SpatialQuery,
) {
    let mut remaining_time = delta_time;
    let mut excluded_entities = vec![*entity];
    excluded_entities.push(*entity);

    for _ in 0..config.max_iterations {
        let wish_motion = velocity.0 * remaining_time;

        if let Some(hit) = spatial_query.cast_shape(
            collider,
            transform.translation,
            transform.rotation,
            Dir3::new(wish_motion.normalize_or_zero()).unwrap_or(Dir3::X),
            &ShapeCastConfig::from_max_distance(wish_motion.length()),
            &SpatialQueryFilter::default()
                .with_excluded_entities(excluded_entities.iter().copied()),
        ) {
            let fraction = hit.distance / wish_motion.length();

            // Move to just before the collision point
            transform.translation += wish_motion.normalize_or_zero() * hit.distance;

            // Prevents sticking
            transform.translation += hit.normal1 * config.epsilon;

            // Project velocity onto the surface plane
            velocity.0 = velocity.0 - (hit.normal1 * velocity.0.dot(hit.normal1));

            // Scale remaining time
            remaining_time *= 1.0 - fraction;
        } else {
            // No collision, move the full remaining distance
            transform.translation += wish_motion;
            break;
        }
    }
}
