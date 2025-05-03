use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, PhysicsDebugPlugin, RigidBody},
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use input::{DefaultContext, InputPlugin, Jump};
use level::LevelPlugin;
use movement::KCCBundle;
mod input;
mod level;
mod movement;

#[derive(Component)]
struct KCCMarker;

#[derive(Component)]
struct DefaultCamera;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        InputPlugin,
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        LevelPlugin,
    ));
    app.add_systems(Startup, setup);
    app.add_systems(Update, movement);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 5., 0.0),
        Actions::<DefaultContext>::default(),
        KCCBundle::default(),
        children![(
            Camera3d::default(),
            DefaultCamera,
            Transform::from_xyz(0.0, 1.5, 0.0)
        )],
    ));
}

fn movement(q_kcc: Query<Entity, With<KCCMarker>>, q_input: Single<&Actions<DefaultContext>>) {
    if q_input.action::<Jump>().state() == ActionState::Fired {
        println!("Jump action fired!");
    }
}
