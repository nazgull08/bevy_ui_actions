//! Drag & drop example.
//!
//! Demonstrates:
//! - Draggable elements
//! - DropTarget zones
//! - OnDragStart, OnDrop, OnDragCancel actions
//! - DragState resource
//!
//! Run: `cargo run --example drag_drop -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .run();
}

// ============ Actions ============

struct LogDragStartAction {
    name: &'static str,
}

impl UiAction for LogDragStartAction {
    fn execute(&self, _world: &mut World) {
        info!("Started dragging: {}", self.name);
    }
}

struct DropIntoSlotAction {
    slot_index: usize,
}

impl UiAction for DropIntoSlotAction {
    fn execute(&self, world: &mut World) {
        let drag_state = world.resource::<DragState>();
        if let Some(dragged_entity) = drag_state.dragging {
            info!(
                "Dropped entity {:?} into slot {}",
                dragged_entity, self.slot_index
            );
        }
    }
}

struct LogDragCancelAction {
    name: &'static str,
}

impl UiAction for LogDragCancelAction {
    fn execute(&self, _world: &mut World) {
        info!("Cancelled drag: {}", self.name);
    }
}

// ============ UI ============

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(40.0),
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Drag & Drop Example"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));

            // Draggable items row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_draggable_item(row, "Item A", Color::srgb(0.8, 0.3, 0.3));
                    spawn_draggable_item(row, "Item B", Color::srgb(0.3, 0.8, 0.3));
                    spawn_draggable_item(row, "Item C", Color::srgb(0.3, 0.3, 0.8));
                });

            // Drop targets row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    for i in 0..4 {
                        spawn_drop_slot(row, i);
                    }
                });

            // Hint
            parent.spawn((
                Text::new("Drag items to slots. Check console for logs."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn spawn_draggable_item(parent: &mut ChildSpawnerCommands, name: &'static str, color: Color) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            Draggable,
            OnDragStart::new(LogDragStartAction { name }),
            OnDragCancel::new(LogDragCancelAction { name }),
            InteractiveVisual,
        ))
        .with_children(|item| {
            item.spawn((
                Text::new(name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ));
        });
}

fn spawn_drop_slot(parent: &mut ChildSpawnerCommands, index: usize) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            DropTarget,
            OnDrop::new(DropIntoSlotAction { slot_index: index }),
        ))
        .with_children(|slot| {
            slot.spawn((
                Text::new(format!("Slot {}", index)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}
