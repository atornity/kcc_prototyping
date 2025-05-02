use avian3d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use input::{DefaultContext, InputPlugin, Jump};
mod input;
mod level;

#[derive(Component)]
struct KCCMarker;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, InputPlugin, PhysicsPlugins::default()));
    app.add_systems(Startup, setup);
    app.add_systems(Update, movement);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 0.0),
        KCCMarker,
        Actions::<DefaultContext>::default(),
    ));
}

fn movement(q_kcc: Query<Entity, With<KCCMarker>>, q_input: Single<&Actions<DefaultContext>>) {
    if q_input.action::<Jump>().state() == ActionState::Fired {
        println!("Jump action fired!");
    }
}
