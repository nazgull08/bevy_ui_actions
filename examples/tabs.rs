//! Tabs example.
//!
//! Demonstrates:
//! - TabGroup, Tab, TabContent
//! - Automatic content visibility switching
//! - Active marker for styling
//! - VisualStyle for custom colors
//!
//! Run: `cargo run --example tabs -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .run();
}

// ============ Setup ============

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
                Text::new("Tabs Example"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));

            // Tab container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.0),
                        ..default()
                    },
                    TabGroup::new(0),
                ))
                .with_children(|tab_group| {
                    // Tab buttons row
                    tab_group
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_tab_button(row, 0, "Inventory", true);
                            spawn_tab_button(row, 1, "Stats", false);
                            spawn_tab_button(row, 2, "Skills", false);
                            spawn_tab_button(row, 3, "Map", false);
                        });

                    // Tab content panels
                    spawn_tab_content(
                        tab_group,
                        0,
                        "Inventory Content",
                        "Your items would be displayed here.\n\n• Sword\n• Shield\n• Potion x5",
                        Color::srgb(0.2, 0.15, 0.15),
                    );

                    spawn_tab_content(
                        tab_group,
                        1,
                        "Character Stats",
                        "STR: 10\nDEX: 8\nINT: 12\nVIT: 9",
                        Color::srgb(0.15, 0.2, 0.15),
                    );

                    spawn_tab_content(
                        tab_group,
                        2,
                        "Skills & Abilities",
                        "Active Skills:\n• Fireball (Lv.3)\n• Heal (Lv.2)\n\nPassive Skills:\n• Critical Hit",
                        Color::srgb(0.15, 0.15, 0.2),
                    );

                    spawn_tab_content(
                        tab_group,
                        3,
                        "World Map",
                        "[Map would be rendered here]\n\nCurrent Location: Town Square\nDiscovered: 12/50 areas",
                        Color::srgb(0.18, 0.18, 0.15),
                    );
                });

            // Hint
            parent.spawn((
                Text::new("Click tabs to switch between panels"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn spawn_tab_button(parent: &mut ChildSpawnerCommands, index: usize, label: &str, is_active: bool) {
    let mut entity = parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect {
                left: Val::Px(1.0),
                right: Val::Px(1.0),
                top: Val::Px(1.0),
                bottom: Val::Px(0.0),
            },
            ..default()
        },
        BackgroundColor(if is_active {
            Color::srgb(0.28, 0.28, 0.32)
        } else {
            Color::srgb(0.15, 0.15, 0.18)
        }),
        BorderColor(Color::srgb(0.3, 0.3, 0.35)),
        Tab::new(index),
        VisualStyle::tab(),
        InteractiveVisual,
        Interaction::None,
    ));

    // First tab is active by default
    if is_active {
        entity.insert(Active);
    }

    entity.with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

fn spawn_tab_content(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    title: &str,
    content: &str,
    bg_color: Color,
) {
    let display = if index == 0 {
        Display::Flex
    } else {
        Display::None
    };

    parent
        .spawn((
            Node {
                display,
                width: Val::Percent(100.0),
                min_height: Val::Px(200.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(Color::srgb(0.3, 0.3, 0.35)),
            TabContent::new(index),
        ))
        .with_children(|panel| {
            // Title
            panel.spawn((
                Text::new(title),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Content
            panel.spawn((
                Text::new(content),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}
