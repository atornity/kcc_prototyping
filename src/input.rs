use bevy::{math::FloatPow, prelude::*, window::CursorGrabMode};
use bevy_enhanced_input::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin);
        app.add_input_context::<DefaultContext>();
        app.add_observer(binding);
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
pub struct Look;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct ToggleCameraMode;

#[derive(InputContext)]
pub struct DefaultContext;

fn binding(
    trigger: Trigger<Binding<DefaultContext>>,
    mut players: Query<&mut Actions<DefaultContext>>,
) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers(DeadZone::default());
    actions.bind::<CaptureCursor>().to(MouseButton::Left);
    actions.bind::<ReleaseCursor>().to(KeyCode::Escape);
    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::East))
        .with_conditions(JustPress::default());
    actions
        .bind::<Look>()
        .to((
            Input::mouse_motion().with_modifiers((Scale::splat(0.05), Negate::all())),
            Axial::right_stick().with_modifiers_each(Negate::x()),
        ))
        .with_modifiers(AtLeast(0.2));
    actions
        .bind::<ToggleCameraMode>()
        .to((KeyCode::F1, GamepadButton::DPadUp))
        .with_conditions(JustPress::default());
}

fn capture_cursor(_trigger: Trigger<Completed<CaptureCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

fn release_cursor(_trigger: Trigger<Completed<ReleaseCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

// InputModifier that sets the value of an input to 0.0 if a threshold is not reached
// Used to prevent stick drift in (Nintendo) controllers
#[derive(Debug)]
struct AtLeast(pub f32);

impl Default for AtLeast {
    fn default() -> Self {
        Self(0.2)
    }
}

impl InputModifier for AtLeast {
    fn apply(
        &mut self,
        _: &bevy_enhanced_input::action_map::ActionMap,
        _: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        // TODO: don't do this
        match value {
            ActionValue::Bool(bool) => ActionValue::Bool(bool),
            ActionValue::Axis1D(vec1) => {
                if vec1 < self.0 {
                    ActionValue::Axis1D(0.0)
                } else {
                    ActionValue::Axis1D(vec1)
                }
            }
            ActionValue::Axis2D(vec2) => {
                if vec2.length_squared() < self.0.squared() {
                    ActionValue::Axis2D(Vec2::splat(0.0))
                } else {
                    ActionValue::Axis2D(vec2)
                }
            }
            ActionValue::Axis3D(vec3) => {
                if vec3.length_squared() < self.0.squared() {
                    ActionValue::Axis3D(Vec3::splat(0.0))
                } else {
                    ActionValue::Axis3D(vec3)
                }
            }
        }
    }
}
