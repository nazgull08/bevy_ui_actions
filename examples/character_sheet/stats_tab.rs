use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

use crate::data::*;

pub fn spawn_stats_tab(parent: &mut ChildSpawnerCommands, stats: &CharacterStats) {
    // Level
    parent.spawn(Node::row(10.0)).with_children(|row| {
        row.ui_text(TextRole::Heading, format!("Level {}", stats.level))
            .insert(LevelText);
    });

    // XP bar
    spawn_xp_bar(parent, stats);

    // Separator
    spawn_separator(parent);

    // Attributes heading + available points
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.ui_text(TextRole::Button, "Attributes");
            row.ui_text_styled(
                format!("Available: {}", stats.available_points),
                14.0,
                Color::srgb(0.8, 0.75, 0.3),
            )
            .insert(PointsText);
        });

    // Attribute rows
    spawn_attribute_row(parent, "Strength", Attribute::Strength, stats);
    spawn_attribute_row(parent, "Dexterity", Attribute::Dexterity, stats);
    spawn_attribute_row(parent, "Intelligence", Attribute::Intelligence, stats);
    spawn_attribute_row(parent, "Vitality", Attribute::Vitality, stats);

    // Respec button
    parent
        .spawn(Node {
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.15, 0.15)),
                BorderColor(Color::srgb(0.5, 0.3, 0.3)),
                OnClick::new(ShowRespecModal),
                InteractiveVisual,
            ))
            .with_children(|btn| {
                btn.ui_text_styled("Reset", 13.0, Color::srgb(0.85, 0.5, 0.5));
            });
        });

    // Derived stats
    spawn_separator(parent);
    parent.ui_text(TextRole::Button, "Derived Stats");

    spawn_derived_row(parent, "Physical Damage", "phys_dmg", stats);
    spawn_derived_row(parent, "Magic Power", "magic_pow", stats);
    spawn_derived_row(parent, "Dodge Chance", "dodge", stats);
    spawn_derived_row(parent, "Max Health", "max_hp", stats);
    spawn_derived_row(parent, "Max Mana", "max_mp", stats);
    spawn_derived_row(parent, "Max Stamina", "max_sp", stats);
}

fn spawn_xp_bar(parent: &mut ChildSpawnerCommands, stats: &CharacterStats) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|row| {
                row.ui_text(TextRole::Label, "Experience");
                row.ui_text(
                    TextRole::Label,
                    format!("{:.0}/{:.0}", stats.xp, stats.xp_max),
                )
                .insert(XpText);
            });

            let xp_config = ProgressBarConfig {
                width: Val::Percent(100.0),
                height: Val::Px(12.0),
                fill_color: Color::srgb(0.7, 0.6, 0.2),
                ..default()
            };
            let bar = col.spawn_progress_bar(xp_config, stats.xp / stats.xp_max);
            col.commands().entity(bar).insert(XpBar);
        });
}

fn spawn_separator(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.25, 0.30)),
    ));
}

pub fn derived_value(key: &str, stats: &CharacterStats) -> String {
    match key {
        "phys_dmg" => format!("{:.0}", 10.0 + stats.strength as f32 * 1.5),
        "magic_pow" => format!("{:.0}", 5.0 + stats.intelligence as f32 * 2.0),
        "dodge" => format!("{:.0}%", stats.dexterity as f32 * 0.5),
        "max_hp" => format!("{:.0}", stats.health_max),
        "max_mp" => format!("{:.0}", stats.mana_max),
        "max_sp" => format!("{:.0}", stats.stamina_max),
        _ => "?".into(),
    }
}

fn spawn_derived_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    key: &'static str,
    stats: &CharacterStats,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            row.ui_text(TextRole::Body, label);
            row.ui_text(TextRole::Body, derived_value(key, stats))
                .insert(DerivedStatText(key));
        });
}

fn spawn_attr_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    bg: Color,
    border: Color,
    text_color: Color,
    action: impl UiAction,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(border),
            OnClick::new(action),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text_styled(label, 14.0, text_color);
        });
}

fn spawn_attribute_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    attr: Attribute,
    stats: &CharacterStats,
) {
    let value = attr.get(stats);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn(Node {
                width: Val::Px(100.0),
                ..default()
            })
            .with_children(|w| {
                w.ui_text(TextRole::Body, label);
            });

            // Bar
            let bar_config = ProgressBarConfig {
                width: Val::Px(120.0),
                height: Val::Px(10.0),
                fill_color: attr.bar_color(),
                ..ProgressBarConfig::attribute()
            };
            let bar = row.spawn_progress_bar(bar_config, value as f32 / 30.0);
            row.commands().entity(bar).insert(AttributeBar(attr));

            // - button
            spawn_attr_button(
                row,
                "-",
                Color::srgb(0.22, 0.18, 0.18),
                Color::srgb(0.4, 0.3, 0.3),
                Color::srgb(0.8, 0.5, 0.5),
                DecrementAttribute(attr),
            );

            // Value text
            row.spawn(Node {
                width: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|w| {
                w.ui_text(TextRole::Body, format!("{}", value))
                    .insert(AttributeValue(attr));
            });

            // + button
            spawn_attr_button(
                row,
                "+",
                Color::srgb(0.18, 0.22, 0.18),
                Color::srgb(0.3, 0.4, 0.3),
                Color::srgb(0.5, 0.8, 0.5),
                IncrementAttribute(attr),
            );
        });
}
