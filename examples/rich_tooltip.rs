//! Rich Tooltip Example
//!
//! Demonstrates:
//! - Simple text tooltips (backwards compatible)
//! - Rich tooltips with sections (title, stats, description)
//! - Stat comparison with diff indicators
//! - Builder pattern for tooltip construction
//!
//! Run: `cargo run --example rich_tooltip -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<EquippedItem>()
        .add_systems(Startup, setup)
        .run();
}

// ============ Mock Item Data ============

#[derive(Clone)]
struct ItemData {
    name: &'static str,
    category: &'static str,
    slot: &'static str,
    damage: Option<f32>,
    defense: Option<f32>,
    speed: Option<f32>,
    description: &'static str,
    weight: f32,
    value: u32,
}

const ITEMS: &[ItemData] = &[
    ItemData {
        name: "Iron Sword",
        category: "Weapon",
        slot: "Main Hand",
        damage: Some(12.0),
        defense: None,
        speed: Some(1.0),
        description: "A reliable iron sword. Standard issue for soldiers.",
        weight: 3.5,
        value: 80,
    },
    ItemData {
        name: "Steel Greatsword",
        category: "Weapon",
        slot: "Two-Handed",
        damage: Some(24.0),
        defense: None,
        speed: Some(0.7),
        description: "A massive blade that requires both hands.",
        weight: 8.0,
        value: 250,
    },
    ItemData {
        name: "Chainmail Vest",
        category: "Armor",
        slot: "Chest",
        damage: None,
        defense: Some(8.0),
        speed: None,
        description: "Interlocking metal rings provide decent protection.",
        weight: 12.0,
        value: 150,
    },
    ItemData {
        name: "Health Potion",
        category: "Consumable",
        slot: "",
        damage: None,
        defense: None,
        speed: None,
        description: "Restores 50 HP instantly.",
        weight: 0.5,
        value: 25,
    },
];

#[derive(Resource)]
struct EquippedItem {
    damage: f32,
    defense: f32,
    speed: f32,
}

impl Default for EquippedItem {
    fn default() -> Self {
        Self {
            damage: 8.0,
            defense: 5.0,
            speed: 1.0,
        }
    }
}

// ============ Setup ============

fn setup(mut commands: Commands, equipped: Res<EquippedItem>) {
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
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Rich Tooltip Example"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Hover over items to see tooltips"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Simple tooltips row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("Simple Tooltips"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(15.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_simple_button(row, "Button A", "Simple text tooltip");
                        spawn_simple_button(row, "Button B", "Another tooltip");
                        spawn_simple_button(row, "Button C", "Short tip");
                    });
                });

            // Rich tooltips row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("Rich Tooltips (Items)"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(15.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for item in ITEMS {
                            spawn_item_slot(row, item, &equipped);
                        }
                    });
                });

            // Hint
            parent.spawn((
                Text::new("Green = better, Red = worse than equipped"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_simple_button(parent: &mut ChildSpawnerCommands, label: &str, tooltip_text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.28)),
            Tooltip::new(tooltip_text),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ));
        });
}

fn spawn_item_slot(parent: &mut ChildSpawnerCommands, item: &ItemData, equipped: &EquippedItem) {
    let tooltip = build_item_tooltip(item, equipped);

    let bg_color = match item.category {
        "Weapon" => Color::srgb(0.35, 0.25, 0.25),
        "Armor" => Color::srgb(0.25, 0.30, 0.35),
        "Consumable" => Color::srgb(0.25, 0.35, 0.25),
        _ => Color::srgb(0.25, 0.25, 0.25),
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(90.0),
                height: Val::Px(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            tooltip,
            InteractiveVisual,
        ))
        .with_children(|slot| {
            slot.spawn((
                Text::new(match item.category {
                    "Weapon" => "⚔",
                    "Armor" => "🛡",
                    "Consumable" => "🧪",
                    _ => "📦",
                }),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
            ));

            slot.spawn((
                Text::new(item.name.split_whitespace().next().unwrap_or(item.name)),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn build_item_tooltip(item: &ItemData, equipped: &EquippedItem) -> Tooltip {
    let mut builder = Tooltip::builder()
        .title(item.name)
        .subtitle(if item.slot.is_empty() {
            item.category.to_string()
        } else {
            format!("{} • {}", item.category, item.slot)
        })
        .separator();

    // Stats with comparison
    if let Some(damage) = item.damage {
        let diff = calc_diff(damage, equipped.damage);
        builder = builder.stat_diff("Damage", format!("{:.0}", damage), diff);
    }

    if let Some(defense) = item.defense {
        let diff = calc_diff(defense, equipped.defense);
        builder = builder.stat_diff("Defense", format!("{:.0}", defense), diff);
    }

    if let Some(speed) = item.speed {
        let diff = calc_diff(speed, equipped.speed);
        builder = builder.stat_diff("Speed", format!("{:.1}x", speed), diff);
    }

    // Description
    builder = builder.separator().text(item.description);

    // Footer
    builder = builder
        .separator()
        .key_value("Weight", format!("{:.1}", item.weight))
        .key_value("Value", format!("{}g", item.value));

    builder.build()
}

fn calc_diff(new: f32, current: f32) -> StatDiff {
    let delta = new - current;
    if delta > 0.01 {
        StatDiff::Better(delta)
    } else if delta < -0.01 {
        StatDiff::Worse(-delta)
    } else {
        StatDiff::Neutral
    }
}
