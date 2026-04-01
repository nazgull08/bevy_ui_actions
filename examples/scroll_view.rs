//! ScrollView example.
//!
//! Demonstrates:
//! - Vertical scroll with mouse wheel
//! - Scrollbar with draggable thumb
//! - spawn_scroll_view (no scrollbar) vs spawn_scroll_view_with (with scrollbar)
//! - Automatic bounds clamping
//!
//! Run: `cargo run --example scroll_view -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<ClickCounter>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_counter_text)
        .run();
}

#[derive(Resource, Default)]
struct ClickCounter(usize);

#[derive(Component)]
struct CounterText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node::centered(20.0))
        .with_children(|root| {
            root.ui_text(TextRole::Heading, "ScrollView Example");
            root.ui_text(
                TextRole::Label,
                "Scroll with mouse wheel. Drag scrollbar thumb. Click buttons inside scroll.",
            );

            // Three scroll views side by side
            root.spawn(Node::row(20.0)).with_children(|row| {
                // Left: no scrollbar (simple API)
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|col| {
                    col.ui_text(TextRole::Button, "Item List (no scrollbar)");

                    col.spawn_scroll_view(ScrollViewConfig {
                        width: Val::Px(250.0),
                        height: Val::Px(300.0),
                        background: Some(Color::srgb(0.1, 0.1, 0.12)),
                        ..default()
                    })
                    .with_children(|scroll| {
                        for i in 0..20 {
                            spawn_list_item(scroll, i);
                        }
                    });
                });

                // Right: with scrollbar (callback API)
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|col| {
                    col.ui_text(TextRole::Button, "Quest Log (with scrollbar)");

                    col.spawn_scroll_view_with(
                        ScrollViewConfig {
                            width: Val::Px(300.0),
                            height: Val::Px(300.0),
                            background: Some(Color::srgb(0.1, 0.1, 0.12)),
                            show_scrollbar: true,
                            ..default()
                        },
                        |scroll| {
                            for (title, desc) in QUEST_ENTRIES {
                                spawn_quest_entry(scroll, title, desc);
                            }
                        },
                    );
                });

                // Right: clickable buttons inside scroll
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|col| {
                    col.ui_text(TextRole::Button, "Clickable (with scrollbar)");

                    col.spawn_scroll_view_with(
                        ScrollViewConfig {
                            width: Val::Px(220.0),
                            height: Val::Px(300.0),
                            background: Some(Color::srgb(0.1, 0.1, 0.12)),
                            show_scrollbar: true,
                            ..default()
                        },
                        |scroll| {
                            for i in 0..15 {
                                spawn_action_button(scroll, i);
                            }
                        },
                    );

                    col.ui_text(TextRole::Body, "Clicked: 0")
                        .insert(CounterText);
                });
            });

            root.ui_text(
                TextRole::Caption,
                "Left: plain list | Center: quest log | Right: clickable buttons",
            );
        });
}

fn spawn_list_item(parent: &mut ChildSpawnerCommands, index: usize) {
    let color = if index % 2 == 0 {
        Color::srgb(0.18, 0.18, 0.22)
    } else {
        Color::srgb(0.15, 0.15, 0.18)
    };

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|item| {
            item.ui_text(
                TextRole::Body,
                format!("Item #{} — Some inventory item", index),
            );
        });
}

const QUEST_ENTRIES: &[(&str, &str)] = &[
    (
        "The Lost Sword",
        "Find the ancient blade hidden in the depths of the dungeon. The blacksmith says it was lost decades ago.",
    ),
    (
        "Spider Infestation",
        "Clear the spider nest in the lower caves. Residents report strange noises at night.",
    ),
    (
        "Deliver the Message",
        "Bring the sealed letter to the guard captain in the eastern tower. Time is of the essence.",
    ),
    (
        "Gather Herbs",
        "Collect 5 moonflower petals from the garden terrace. The alchemist needs them for a healing potion.",
    ),
    (
        "The Missing Guard",
        "Investigate the disappearance of the night watch guard. He was last seen near the storage rooms.",
    ),
    (
        "Ancient Mechanism",
        "Discover how to operate the strange device found in the underground chamber.",
    ),
    (
        "Repair the Bridge",
        "Find materials to fix the collapsed bridge on level 3. Wood planks and rope should suffice.",
    ),
    (
        "The Sealed Door",
        "Find a way to open the sealed door in the cathedral hall. There must be a key somewhere.",
    ),
];

// ============ Clickable buttons ============

struct ButtonClickAction {
    index: usize,
}

impl UiAction for ButtonClickAction {
    fn execute(&self, world: &mut World) {
        world.resource_mut::<ClickCounter>().0 += 1;
        info!("Clicked button #{}", self.index);
    }
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, index: usize) {
    let hue = (index as f32 * 25.0) % 360.0;
    let color = Color::hsl(hue, 0.4, 0.25);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            OnClick::new(ButtonClickAction { index }),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text(TextRole::Button, format!("Action #{}", index));
        });
}

fn update_counter_text(counter: Res<ClickCounter>, mut query: Query<&mut Text, With<CounterText>>) {
    if counter.is_changed() {
        for mut text in &mut query {
            **text = format!("Clicked: {}", counter.0);
        }
    }
}

// ============ Quest entries ============

fn spawn_quest_entry(parent: &mut ChildSpawnerCommands, title: &str, description: &str) {
    parent
        .spawn_panel(PanelConfig {
            background: Color::srgb(0.14, 0.14, 0.17),
            border_color: Color::srgb(0.25, 0.25, 0.30),
            ..PanelConfig::dark()
        })
        .with_children(|panel| {
            panel.ui_text(TextRole::Button, title);
            panel.ui_text(TextRole::Body, description);
        });
}
