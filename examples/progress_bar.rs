//! Progress bar widget example.
//!
//! Demonstrates:
//! - ProgressBar with different styles (HP, MP, SP, attributes)
//! - Dynamic value updates
//! - SpawnProgressBarExt helper
//! - UiTextExt for labels
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
        .spawn(Node::centered(20.0))
        .with_children(|parent| {
            parent.ui_text(TextRole::Heading, "Progress Bar Example");

            // Stats panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|panel| {
                    spawn_labeled_bar(
                        panel,
                        "Health",
                        ProgressBarConfig::health(),
                        0.75,
                        HealthBar,
                    );
                    spawn_labeled_bar(panel, "Mana", ProgressBarConfig::mana(), 0.5, ManaBar);
                    spawn_labeled_bar(
                        panel,
                        "Stamina",
                        ProgressBarConfig::stamina(),
                        0.6,
                        StaminaBar,
                    );
                    spawn_labeled_bar(
                        panel,
                        "Strength",
                        ProgressBarConfig {
                            width: Val::Px(200.0),
                            ..ProgressBarConfig::attribute()
                        },
                        8.0 / 30.0,
                        StrengthBar,
                    );
                });

            // Current values text
            parent
                .ui_text(TextRole::Body, "")
                .insert(StatsText);

            // Controls hint
            parent.ui_text(
                TextRole::Caption,
                "Controls: Q/W - Health | A/S - Mana | Z/X - Stamina | 1/2 - Strength",
            );
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
            row.ui_text(TextRole::Button, format!("{:8}", label))
                .insert(Node {
                    width: Val::Px(80.0),
                    ..default()
                });

            let bar_entity = row.spawn_progress_bar(config, initial);
            row.commands().entity(bar_entity).insert(marker);
        });
}

// ============ Input ============

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mut stats: ResMut<PlayerStats>) {
    if keys.just_pressed(KeyCode::KeyQ) {
        stats.health = (stats.health - 10.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyW) {
        stats.health = (stats.health + 10.0).min(stats.health_max);
    }
    if keys.just_pressed(KeyCode::KeyA) {
        stats.mana = (stats.mana - 15.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        stats.mana = (stats.mana + 15.0).min(stats.mana_max);
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        stats.stamina = (stats.stamina - 20.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        stats.stamina = (stats.stamina + 20.0).min(stats.stamina_max);
    }
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
