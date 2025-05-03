use bevy::{prelude::*, window::CursorGrabMode};
use bevy_enhanced_input::prelude::*;

use crate::{DefaultCamera, KCCMarker};

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin);
        app.add_input_context::<DefaultContext>();
        app.add_observer(binding);
        app.add_observer(rotate);
        app.add_observer(capture_cursor);
        app.add_observer(release_cursor);
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Jump;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct CaptureCursor;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct ReleaseCursor;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Rotate;

#[derive(InputContext)]
pub struct DefaultContext;

// To define bindings for actions, write an observer for `Binding`.
// It's also possible to create bindings before the insertion,
// but this way you can conveniently reload bindings when settings change.
fn binding(
    trigger: Trigger<Binding<DefaultContext>>,
    mut players: Query<&mut Actions<DefaultContext>>,
) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    // Bindings like WASD or sticks are very common,
    // so we provide built-ins to assign all keys/axes at once.
    // We don't assign any conditions and in this case the action will
    // be triggered with any non-zero value.
    // An action can have multiple inputs bound to it
    // and will respond to any of them.
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(0.3), // Additionally multiply by a constant to achieve the desired speed.
        ));

    actions.bind::<Rotate>().to((
        // You can attach modifiers to individual inputs as well.
        Input::mouse_motion().with_modifiers((Scale::splat(0.1), Negate::all())),
        Axial::right_stick().with_modifiers_each((Scale::splat(2.0), Negate::x())),
    ));

    actions.bind::<CaptureCursor>().to(MouseButton::Left);
    actions.bind::<ReleaseCursor>().to(KeyCode::Escape);

    actions.bind::<Jump>().to(KeyCode::Space);
}

fn rotate(
    trigger: Trigger<Fired<Rotate>>,
    mut cameras: Query<&mut Transform, (With<DefaultCamera>)>,
    mut players: Query<&mut Transform, (With<KCCMarker>, Without<DefaultCamera>)>,
) {
    // Get the player rotation (only YAW is applied to the player)
    let mut player_transform = players.get_mut(trigger.target()).unwrap();

    // Get the camera (child of the player) and apply both YAW and PITCH
    let Some(mut camera_transform) = cameras.single_mut().ok() else {
        warn!("No camera found for rotation. Skipping camera rotation.");
        return;
    }; // assuming only one camera exists

    // Extract current yaw and pitch from the camera
    let (mut cam_yaw, mut cam_pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    // Apply rotation deltas from input
    cam_yaw += trigger.value.x.to_radians(); // Yaw (left-right)
    cam_pitch += trigger.value.y.to_radians(); // Pitch (up-down)

    // Clamp pitch to prevent flipping (optional but recommended)
    cam_pitch = cam_pitch.clamp(-1.5, 1.5); // ~±85 degrees

    // Set camera rotation (yaw and pitch)
    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, cam_yaw, cam_pitch, 0.0);

    // Set player rotation (yaw only, pitch stays 0)
    player_transform.rotation = Quat::from_euler(EulerRot::YXZ, cam_yaw, 0.0, 0.0);
}

fn capture_cursor(_trigger: Trigger<Completed<CaptureCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

fn release_cursor(_trigger: Trigger<Completed<ReleaseCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}
