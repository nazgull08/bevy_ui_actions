//! Minimal example — counter with buttons.
//!
//! Demonstrates:
//! - OnClick actions
//! - OnHover/OnPress actions (logging)
//! - InteractiveVisual for automatic feedback
//! - UiTextExt for text spawning
//!
//! Run: `cargo run --example counter -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<Counter>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_counter_text)
        .run();
}

#[derive(Resource, Default)]
struct Counter(i32);

// ============ Click Actions ============

struct IncrementAction;

impl UiAction for IncrementAction {
    fn execute(&self, world: &mut World) {
        world.resource_mut::<Counter>().0 += 1;
    }
}

struct DecrementAction;

impl UiAction for DecrementAction {
    fn execute(&self, world: &mut World) {
        world.resource_mut::<Counter>().0 -= 1;
    }
}

// ============ Hover/Press Actions ============

struct LogHoverAction {
    name: &'static str,
}

impl UiAction for LogHoverAction {
    fn execute(&self, _world: &mut World) {
        info!("Hovered over: {}", self.name);
    }
}

struct LogPressAction {
    name: &'static str,
}

impl UiAction for LogPressAction {
    fn execute(&self, _world: &mut World) {
        info!("Pressed: {}", self.name);
    }
}

// ============ UI ============

#[derive(Component)]
struct CounterText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            // Counter text
            parent
                .ui_text_styled("0", 48.0, Color::WHITE)
                .insert(CounterText);

            // Buttons row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    // Decrement button with hover/press logging
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        OnClick::new(DecrementAction),
                        OnHover::new(LogHoverAction { name: "Decrement" }),
                        OnPress::new(LogPressAction { name: "Decrement" }),
                        InteractiveVisual,
                    ))
                    .with_children(|btn| {
                        btn.ui_text_styled("-", 32.0, Color::WHITE);
                    });

                    // Increment button — using helper
                    row.spawn_button(IncrementAction, "+");
                });

            // Hint
            parent.ui_text(TextRole::Caption, "Check console for hover/press logs on '-' button");
        });
}

fn update_counter_text(counter: Res<Counter>, mut query: Query<&mut Text, With<CounterText>>) {
    if counter.is_changed() {
        for mut text in &mut query {
            **text = counter.0.to_string();
        }
    }
}
