pub mod fly_camera;
pub mod orbit_camera;

use std::f32::consts::PI;

use crate::{
    Attachments,
    input::{DefaultContext, Look, ToggleFlyCam, ToggleViewPerspective},
    movement::Frozen,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use fly_camera::{FlySpeed, FlyingCamera};
use orbit_camera::{FirstPersonCamera, FollowOrigin, SpringArm};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((fly_camera::plugin, orbit_camera::plugin));
        app.add_systems(Update, view_input);
        app.add_observer(toggle_cam_perspective);
        app.add_observer(toggle_fly_cam);
    }
}

#[derive(Component)]
#[require(Camera3d, Sensitivity, ViewAngles, FollowOrigin, SpringArm, FlySpeed)]
pub struct MainCamera;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Sensitivity(pub f32);

impl Default for Sensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub(crate) struct ViewAngles {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl ViewAngles {
    pub fn to_quat(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
}

fn toggle_cam_perspective(
    trigger: Trigger<Fired<ToggleViewPerspective>>,
    mut commands: Commands,
    query: Query<&Attachments>,
    cameras: Query<(Entity, Has<FirstPersonCamera>), With<Camera>>,
) -> Result {
    let attachments = query.get(trigger.target())?;

    for (camera, first_person) in cameras.iter_many(attachments.iter()) {
        match first_person {
            true => commands.entity(camera).remove::<FirstPersonCamera>(),
            false => commands.entity(camera).insert(FirstPersonCamera),
        };
    }

    Ok(())
}

fn toggle_fly_cam(
    trigger: Trigger<Fired<ToggleFlyCam>>,
    mut commands: Commands,
    mut query: Query<&Attachments>,
    cameras: Query<(Entity, Has<FlyingCamera>), With<Camera>>,
) -> Result {
    let attachments = query.get_mut(trigger.target())?;

    for (camera, fly_camera) in cameras.iter_many(attachments.iter()) {
        match fly_camera {
            true => {
                commands.entity(trigger.target()).remove::<Frozen>();
                commands
                    .entity(camera)
                    .remove::<FlyingCamera>()
                    .insert(FollowOrigin::default());
            }
            false => {
                commands.entity(trigger.target()).insert(Frozen);
                commands
                    .entity(camera)
                    .remove::<FollowOrigin>()
                    .insert(FlyingCamera);
            }
        };
    }

    Ok(())
}

pub(crate) fn view_input(
    mut cameras: Query<(&mut ViewAngles, &mut Transform, &Sensitivity)>,
    actions: Single<&Actions<DefaultContext>>,
    time: Res<Time>,
) {
    let actions = actions.into_inner();

    for (mut angles, mut transform, sensitivity) in &mut cameras {
        let orbit_input = actions.action::<Look>().value().as_axis2d() * sensitivity.0;
        let angle_deltas = orbit_input * PI * time.delta_secs();

        angles.pitch += angle_deltas.y;
        angles.pitch = angles.pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
        angles.yaw += angle_deltas.x;

        transform.rotation = angles.to_quat();
    }
}
