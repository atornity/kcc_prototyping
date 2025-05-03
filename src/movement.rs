use std::f32::consts::PI;

use avian3d::prelude::{Collider, RigidBody};
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
pub struct KinematicVelocity;

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
            kinematic_velocity: KinematicVelocity,
        }
    }
}

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;

fn movement(
    mut q_kcc: Query<(Entity, &mut Transform), With<KCCMarker>>,
    q_input: Single<&Actions<DefaultContext>>,
    q_camera: Query<&Transform, (With<DefaultCamera>, Without<KCCMarker>)>,
    time: Res<Time>,
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

    let Some((_, mut kcc_transform)) = q_kcc.single_mut().ok() else {
        warn!("No KCC found!");
        return;
    };

    // Rotate the movement direction vector by the camera's yaw
    // movement_dir = Quat::from_rotation_y(camera_yaw) * movement_dir;
    let direction = kcc_transform
        .rotation
        .mul_vec3(Vec3::new(input_vec.x, 0.0, -input_vec.y))
        .normalize_or_zero();

    // Apply the final movement
    kcc_transform.translation += direction * EXAMPLE_MOVEMENT_SPEED * time.delta_secs();
}
