use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin);
        app.add_input_context::<DefaultContext>();
        app.add_observer(binding);
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
