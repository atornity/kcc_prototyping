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

#[derive(Bundle)]
pub struct KCCBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    kcc_marker: KCCMarker,
}

impl Default for KCCBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(0.35, 1.0),
            rigid_body: RigidBody::Kinematic,
            kcc_marker: KCCMarker,
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

    // Create the initial 3D movement direction vector in the XZ plane
    // Map input X (left/right) to world X
    // Map input Y (forward/backward) to world Z
    // Keep world Y (up/down) as 0.0 for planar movement relative to camera yaw
    let mut movement_dir = Vec3::new(input_vec.x, 0.0, -input_vec.y);

    movement_dir = movement_dir.normalize_or_zero();

    println!(
        "Initial Movement direction (before yaw): {:?}",
        movement_dir
    );
    // Get camera yaw (rotation around the Y axis)
    let (camera_yaw, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    // Rotate the movement direction vector by the camera's yaw
    // movement_dir = Quat::from_rotation_y(camera_yaw) * movement_dir;

    // println!("Movement direction after yaw: {:?}", movement_dir); // Renamed for clarity

    let Some((kcc_entity, mut kcc_transform)) = q_kcc.single_mut().ok() else {
        warn!("No KCC found!");
        return;
    };

    // Apply the final movement
    kcc_transform.translation += movement_dir * EXAMPLE_MOVEMENT_SPEED * time.delta_secs();
}
