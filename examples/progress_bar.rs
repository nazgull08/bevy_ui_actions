//! Progress bar widget example.
//!
//! Demonstrates:
//! - ProgressBar with different styles (HP, MP, SP, attributes)
//! - Dynamic value updates
//! - SpawnProgressBarExt helper
//!
//! Run: `cargo run --example progress_bar -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<PlayerStats>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, sync_bars))
        .run();
}

// ============ Data ============

#[derive(Resource)]
struct PlayerStats {
    health: f32,
    health_max: f32,
    mana: f32,
    mana_max: f32,
    stamina: f32,
    stamina_max: f32,
    strength: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 75.0,
            health_max: 100.0,
            mana: 40.0,
            mana_max: 80.0,
            stamina: 60.0,
            stamina_max: 100.0,
            strength: 8,
        }
    }
}

// ============ Markers ============

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct ManaBar;

#[derive(Component)]
struct StaminaBar;

#[derive(Component)]
struct StrengthBar;

#[derive(Component)]
struct StatsText;

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
                Text::new("Progress Bar Example"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));

            // Stats panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|panel| {
                    // HP bar
                    spawn_labeled_bar(
                        panel,
                        "Health",
                        ProgressBarConfig::health(),
                        0.75,
                        HealthBar,
                    );

                    // MP bar
                    spawn_labeled_bar(panel, "Mana", ProgressBarConfig::mana(), 0.5, ManaBar);

                    // SP bar
                    spawn_labeled_bar(
                        panel,
                        "Stamina",
                        ProgressBarConfig::stamina(),
                        0.6,
                        StaminaBar,
                    );

                    // Attribute bar
                    spawn_labeled_bar(
                        panel,
                        "Strength",
                        ProgressBarConfig {
                            width: Val::Px(200.0),
                            ..ProgressBarConfig::attribute()
                        },
                        8.0 / 30.0, // 8 out of 30
                        StrengthBar,
                    );
                });

            // Current values text
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                StatsText,
            ));

            // Controls hint
            parent.spawn((
                Text::new("Controls: Q/W - Health | A/S - Mana | Z/X - Stamina | 1/2 - Strength"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn spawn_labeled_bar<M: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    config: ProgressBarConfig,
    initial: f32,
    marker: M,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(format!("{:8}", label)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    width: Val::Px(80.0),
                    ..default()
                },
            ));

            // Progress bar with marker
            let bar_entity = row.spawn_progress_bar(config, initial);
            row.commands().entity(bar_entity).insert(marker);
        });
}

// ============ Input ============

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mut stats: ResMut<PlayerStats>) {
    // Health: Q to damage, W to heal
    if keys.just_pressed(KeyCode::KeyQ) {
        stats.health = (stats.health - 10.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyW) {
        stats.health = (stats.health + 10.0).min(stats.health_max);
    }

    // Mana: A to spend, S to restore
    if keys.just_pressed(KeyCode::KeyA) {
        stats.mana = (stats.mana - 15.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        stats.mana = (stats.mana + 15.0).min(stats.mana_max);
    }

    // Stamina: Z to drain, X to recover
    if keys.just_pressed(KeyCode::KeyZ) {
        stats.stamina = (stats.stamina - 20.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        stats.stamina = (stats.stamina + 20.0).min(stats.stamina_max);
    }

    // Strength: 1 to decrease, 2 to increase
    if keys.just_pressed(KeyCode::Digit1) && stats.strength > 0 {
        stats.strength -= 1;
    }
    if keys.just_pressed(KeyCode::Digit2) && stats.strength < 30 {
        stats.strength += 1;
    }
}

// ============ Sync ============

fn sync_bars(
    stats: Res<PlayerStats>,
    mut hp_query: Query<
        &mut ProgressBar,
        (
            With<HealthBar>,
            Without<ManaBar>,
            Without<StaminaBar>,
            Without<StrengthBar>,
        ),
    >,
    mut mp_query: Query<
        &mut ProgressBar,
        (
            With<ManaBar>,
            Without<HealthBar>,
            Without<StaminaBar>,
            Without<StrengthBar>,
        ),
    >,
    mut sp_query: Query<
        &mut ProgressBar,
        (
            With<StaminaBar>,
            Without<HealthBar>,
            Without<ManaBar>,
            Without<StrengthBar>,
        ),
    >,
    mut str_query: Query<
        &mut ProgressBar,
        (
            With<StrengthBar>,
            Without<HealthBar>,
            Without<ManaBar>,
            Without<StaminaBar>,
        ),
    >,
    mut text_query: Query<&mut Text, With<StatsText>>,
) {
    if !stats.is_changed() {
        return;
    }

    if let Ok(mut bar) = hp_query.get_single_mut() {
        bar.set(stats.health / stats.health_max);
    }

    if let Ok(mut bar) = mp_query.get_single_mut() {
        bar.set(stats.mana / stats.mana_max);
    }

    if let Ok(mut bar) = sp_query.get_single_mut() {
        bar.set(stats.stamina / stats.stamina_max);
    }

    if let Ok(mut bar) = str_query.get_single_mut() {
        bar.set(stats.strength as f32 / 30.0);
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        **text = format!(
            "HP: {:.0}/{:.0}  MP: {:.0}/{:.0}  SP: {:.0}/{:.0}  STR: {}/30",
            stats.health,
            stats.health_max,
            stats.mana,
            stats.mana_max,
            stats.stamina,
            stats.stamina_max,
            stats.strength
        );
    }
}
