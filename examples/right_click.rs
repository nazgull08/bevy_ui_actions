//! Right-click example (context menu pattern).
//!
//! Demonstrates:
//! - OnClick for left-click
//! - OnRightClick for right-click
//! - Different actions on the same element
//!
//! Run: `cargo run --example right_click -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<ActionLog>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_log_text)
        .run();
}

#[derive(Resource, Default)]
struct ActionLog {
    messages: Vec<String>,
}

impl ActionLog {
    fn add(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }
}

// ============ Actions ============

struct LeftClickAction {
    name: &'static str,
}

impl UiAction for LeftClickAction {
    fn execute(&self, world: &mut World) {
        world
            .resource_mut::<ActionLog>()
            .add(format!("Left clicked: {}", self.name));
    }
}

struct RightClickAction {
    name: &'static str,
}

impl UiAction for RightClickAction {
    fn execute(&self, world: &mut World) {
        world
            .resource_mut::<ActionLog>()
            .add(format!("Right clicked: {} (context menu)", self.name));
    }
}

// ============ UI ============

#[derive(Component)]
struct LogText;

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
            // Title
            parent.spawn((
                Text::new("Right Click Example"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));

            // Items row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_item(row, "Sword", Color::srgb(0.7, 0.5, 0.3));
                    spawn_item(row, "Shield", Color::srgb(0.4, 0.4, 0.6));
                    spawn_item(row, "Potion", Color::srgb(0.3, 0.7, 0.4));
                });

            // Hint
            parent.spawn((
                Text::new("Left click to select, Right click for context menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Log display
            parent.spawn((
                Text::new("Actions will appear here..."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
                LogText,
            ));
        });
}

fn spawn_item(parent: &mut ChildSpawnerCommands, name: &'static str, color: Color) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            OnClick::new(LeftClickAction { name }),
            OnRightClick::new(RightClickAction { name }),
            InteractiveVisual,
        ))
        .with_children(|item| {
            item.spawn((
                Text::new(name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
            ));
        });
}

fn update_log_text(log: Res<ActionLog>, mut query: Query<&mut Text, With<LogText>>) {
    if log.is_changed() {
        for mut text in &mut query {
            if log.messages.is_empty() {
                **text = "Actions will appear here...".to_string();
            } else {
                **text = log.messages.join("\n");
            }
        }
    }
}
