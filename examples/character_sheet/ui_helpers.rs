use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

use crate::data::*;

// -- Tab helpers --

pub fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    label: &str,
    is_active: bool,
) {
    let mut entity = parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 1.0,
            border: UiRect {
                left: Val::Px(1.0),
                right: Val::Px(1.0),
                top: Val::Px(1.0),
                bottom: Val::Px(0.0),
            },
            ..default()
        },
        BackgroundColor(if is_active {
            Color::srgb(0.20, 0.20, 0.24)
        } else {
            Color::srgb(0.12, 0.12, 0.15)
        }),
        BorderColor(Color::srgb(0.3, 0.3, 0.35)),
        Tab::new(index),
        VisualStyle::tab(),
        InteractiveVisual,
        Interaction::None,
    ));

    if is_active {
        entity.insert(Active);
    }

    entity.with_children(|btn| {
        btn.ui_text(TextRole::Button, label);
    });
}

pub fn spawn_tab_panel(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    visible: bool,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                display: if visible { Display::Flex } else { Display::None },
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.10, 0.13)),
            BorderColor(Color::srgb(0.25, 0.25, 0.30)),
            TabContent::new(index),
        ))
        .with_children(content);
}

// -- Stat bar (left panel) --

pub enum StatKind {
    Health,
    Mana,
    Stamina,
}

pub fn spawn_stat_bar(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    stats: &CharacterStats,
    kind: StatKind,
) {
    let (value, max, config) = match kind {
        StatKind::Health => (
            stats.health,
            stats.health_max,
            ProgressBarConfig {
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                ..ProgressBarConfig::health()
            },
        ),
        StatKind::Mana => (
            stats.mana,
            stats.mana_max,
            ProgressBarConfig {
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                ..ProgressBarConfig::mana()
            },
        ),
        StatKind::Stamina => (
            stats.stamina,
            stats.stamina_max,
            ProgressBarConfig {
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                ..ProgressBarConfig::stamina()
            },
        ),
    };

    let ratio = value / max;

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
                row.ui_text(TextRole::Label, label);

                let mut text_entity =
                    row.ui_text(TextRole::Label, format!("{:.0}/{:.0}", value, max));

                match kind {
                    StatKind::Health => {
                        text_entity.insert(HealthText);
                    }
                    StatKind::Mana => {
                        text_entity.insert(ManaText);
                    }
                    StatKind::Stamina => {
                        text_entity.insert(StaminaText);
                    }
                };
            });

            let bar_entity = col.spawn_progress_bar(config, ratio);
            match kind {
                StatKind::Health => {
                    col.commands().entity(bar_entity).insert(HealthBar);
                }
                StatKind::Mana => {
                    col.commands().entity(bar_entity).insert(ManaBar);
                }
                StatKind::Stamina => {
                    col.commands().entity(bar_entity).insert(StaminaBar);
                }
            }
        });
}
