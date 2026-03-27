//! Selection example with BorderStyle.
//!
//! Demonstrates:
//! - `Selected` marker component
//! - `BorderStyle` for border feedback
//! - Combined background + border visual feedback
//! - Single selection in a grid
//!
//! Run: `cargo run --example selection -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<SelectedSlot>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_selection_info)
        .run();
}

// ============================================================
// Resources
// ============================================================

/// Tracks which slot is currently selected
#[derive(Resource, Default)]
struct SelectedSlot {
    entity: Option<Entity>,
    index: Option<usize>,
}

// ============================================================
// Components
// ============================================================

#[derive(Component)]
struct Slot {
    index: usize,
    has_item: bool,
}

#[derive(Component)]
struct SelectionInfoText;

// ============================================================
// Actions
// ============================================================

struct SelectSlotAction {
    index: usize,
}

impl UiAction for SelectSlotAction {
    fn execute(&self, world: &mut World) {
        // Find clicked slot entity
        let clicked_entity = {
            let mut query = world.query::<(Entity, &Slot)>();
            query
                .iter(world)
                .find(|(_, slot)| slot.index == self.index)
                .map(|(e, _)| e)
        };

        let Some(clicked) = clicked_entity else {
            return;
        };

        // Get current selection
        let current_selected = world.resource::<SelectedSlot>().entity;

        // If clicking same slot, deselect
        if current_selected == Some(clicked) {
            world.entity_mut(clicked).remove::<Selected>();
            let mut selected = world.resource_mut::<SelectedSlot>();
            selected.entity = None;
            selected.index = None;
            info!("Deselected slot {}", self.index);
            return;
        }

        // Remove selection from previous slot
        if let Some(prev) = current_selected {
            world.entity_mut(prev).remove::<Selected>();
        }

        // Select new slot
        world.entity_mut(clicked).insert(Selected);
        let mut selected = world.resource_mut::<SelectedSlot>();
        selected.entity = Some(clicked);
        selected.index = Some(self.index);
        info!("Selected slot {}", self.index);
    }
}

// ============================================================
// Setup
// ============================================================

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(30.0),
            ..default()
        })
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("Selection + BorderStyle Example"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            ));

            // Instructions
            root.spawn((
                Text::new("Click slots to select. Click again to deselect."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Slot grid
            root.spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(80.0); 4],
                grid_template_rows: vec![GridTrack::px(80.0); 2],
                row_gap: Val::Px(8.0),
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|grid| {
                for i in 0..8 {
                    let has_item = i < 3; // First 3 slots have items
                    spawn_slot(grid, i, has_item);
                }
            });

            // Selection info panel
            root.spawn((
                Node {
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    min_width: Val::Px(300.0),
                    min_height: Val::Px(80.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor(Color::srgb(0.3, 0.3, 0.35)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("No slot selected"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    SelectionInfoText,
                ));
            });
        });
}

fn spawn_slot(parent: &mut ChildSpawnerCommands, index: usize, has_item: bool) {
    let bg_color = if has_item {
        Color::srgb(0.25, 0.25, 0.30)
    } else {
        Color::srgb(0.15, 0.15, 0.18)
    };

    parent
        .spawn((
            Node {
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(Color::srgb(0.35, 0.35, 0.40)),
            Slot { index, has_item },
            // Visual feedback components
            InteractiveVisual,
            VisualStyle::slot(),
            BorderStyle::slot(),
            // Interactions
            OnClick::new(SelectSlotAction { index }),
            Interaction::None,
        ))
        .with_children(|slot| {
            if has_item {
                // Item icon placeholder
                slot.spawn((
                    Node {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        ..default()
                    },
                    BackgroundColor(item_color(index)),
                ));
            } else {
                // Empty slot label
                slot.spawn((
                    Text::new(format!("{}", index)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                ));
            }
        });
}

fn item_color(index: usize) -> Color {
    match index % 3 {
        0 => Color::srgb(0.7, 0.3, 0.3), // Red
        1 => Color::srgb(0.3, 0.7, 0.3), // Green
        _ => Color::srgb(0.3, 0.3, 0.7), // Blue
    }
}

// ============================================================
// Systems
// ============================================================

fn update_selection_info(
    selected: Res<SelectedSlot>,
    slots: Query<&Slot>,
    mut text: Query<&mut Text, With<SelectionInfoText>>,
) {
    if !selected.is_changed() {
        return;
    }

    let Ok(mut text) = text.single_mut() else {
        return;
    };

    match (selected.entity, selected.index) {
        (Some(entity), Some(index)) => {
            if let Ok(slot) = slots.get(entity) {
                let item_info = if slot.has_item {
                    format!("Item in slot {}", index)
                } else {
                    "Empty slot".to_string()
                };
                text.0 = format!("Selected: Slot {}\n{}", index, item_info);
            }
        }
        _ => {
            text.0 = "No slot selected\n".to_string();
        }
    }
}
