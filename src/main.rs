use avian3d::{
    PhysicsPlugins,
    prelude::{PhysicsDebugPlugin, PhysicsDiagnosticsPlugin, PhysicsDiagnosticsUiPlugin},
};
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use bevy_enhanced_input::prelude::Actions;
use kcc_prototype::{
    Attachments,
    camera::FollowOffset,
    camera::{CameraPlugin, MainCamera},
    character::*,
    input::{DefaultContext, InputPlugin},
    input::{FlyCameraContext, OrbitCameraContext},
    level::LevelGeneratorPlugin,
    movement::{Character, KCCPlugin},
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            InputPlugin,
            CameraPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            LevelGeneratorPlugin,
            KCCPlugin,
            PhysicsDiagnosticsPlugin,
            PhysicsDiagnosticsUiPlugin,
        ))
        .add_systems(Startup, setup)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 10.5, 0.0),
        Actions::<DefaultContext>::default(),
        Actions::<FlyCameraContext>::default(),
        Actions::<OrbitCameraContext>::default(),
        Character::default(),
        Mesh3d(meshes.add(Capsule3d::new(
            EXAMPLE_CHARACTER_RADIUS,
            EXAMPLE_CHARACTER_CAPSULE_LENGTH,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.25),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        })),
        Attachments::spawn_one((
            MainCamera,
            FollowOffset {
                absolute: Vec3::Y * 0.75,
                ..Default::default()
            },
            Camera {
                hdr: true,
                ..Default::default()
            },
            Camera3d::default(),
            Atmosphere::EARTH,
            Exposure::SUNLIGHT,
            Projection::Perspective(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..Default::default()
            }),
            AmbientLight {
                brightness: lux::AMBIENT_DAYLIGHT,
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.5, 0.0),
        )),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
