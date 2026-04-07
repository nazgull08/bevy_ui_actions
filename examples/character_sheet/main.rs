//! Character Sheet — comprehensive showcase of bevy_ui_actions widgets.
//!
//! Demonstrates: Viewport3d, ProgressBar, Panel, Tabs, Drag&Drop, ScrollView,
//! ListView, Rich Tooltips, Modal, DialogueBox, HyperText, InteractiveVisual.
//!
//! Run: `cargo run --example character_sheet -p bevy_ui_actions --features viewport3d`

mod data;
mod equipment_tab;
mod lore_tab;
mod mannequin;
mod stats_tab;
mod ui_helpers;

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

use data::*;
use equipment_tab::{highlight_compatible_slots, spawn_equipment_tab, sync_equipment};
use mannequin::{spawn_mannequin, sync_mannequin_equipment, resolve_mannequin, propagate_render_layers, MannequinParts};
use lore_tab::{setup_lore_registry, spawn_lore_tab};
use stats_tab::spawn_stats_tab;
use ui_helpers::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Character Sheet — bevy_ui_actions showcase".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiActionsPlugin)
        .insert_resource(UiTheme::default().with_offset(2.0))
        .init_resource::<CharacterStats>()
        .init_resource::<PreviousStats>()
        .init_resource::<EquipmentState>()
        .init_resource::<InventoryState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                resolve_mannequin.run_if(not(resource_exists::<MannequinParts>)),
                propagate_render_layers.run_if(resource_exists::<MannequinParts>),
                sync_bars,
                sync_attributes,
                sync_equipment,
                sync_mannequin_equipment.run_if(resource_exists::<MannequinParts>),
                fade_change_flash,
                highlight_compatible_slots,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    stats: Res<CharacterStats>,
    equip: Res<EquipmentState>,
    inv: Res<InventoryState>,
    mut prev_stats: ResMut<PreviousStats>,
) {
    *prev_stats = PreviousStats::snapshot(&stats);
    setup_lore_registry(&mut commands);
    commands.spawn(Camera2d);

    // -- Viewport3d --
    let viewport_config = Viewport3dConfig {
        size: UVec2::new(300, 400),
        render_layer: 1,
        camera_distance: 2.2,
        camera_height: 0.6,
        camera_target: Vec3::new(0.0, 0.3, 0.0),
        rotatable: true,
        rotate_sensitivity: 0.3,
        background: Color::linear_rgba(0.06, 0.06, 0.08, 1.0),
        light_intensity: 100000.0,
        light_offset: Vec3::new(1.0, 1.5, 2.5),
        ..default()
    };
    let viewport = commands.spawn_viewport3d(&viewport_config, &mut images);

    spawn_mannequin(
        &mut commands,
        &asset_server,
        viewport.pivot,
        viewport.render_layer.clone(),
    );

    // -- UI Layout --
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(16.0)),
            column_gap: Val::Px(16.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|root| {
            // ======== Left panel: viewport + bars ========
            root.spawn_panel(PanelConfig {
                width: Val::Px(332.0),
                height: Val::Percent(100.0),
                gap: 12.0,
                padding: 16.0,
                direction: FlexDirection::Column,
                ..PanelConfig::dark()
            })
            .with_children(|left| {
                left.ui_text(TextRole::Heading, "Character");

                left.spawn(Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .add_child(viewport.ui_entity);

                left.ui_text_styled(
                    "\u{21C4} Drag to rotate",
                    12.0,
                    Color::srgb(0.45, 0.45, 0.5),
                );

                spawn_stat_bar(left, "HP", &stats, StatKind::Health);
                spawn_stat_bar(left, "MP", &stats, StatKind::Mana);
                spawn_stat_bar(left, "SP", &stats, StatKind::Stamina);
            });

            // ======== Right panel: tabs ========
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    ..default()
                },
                TabGroup::new(0),
            ))
            .with_children(|tab_group| {
                tab_group
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_tab_button(row, 0, "Stats", true);
                        spawn_tab_button(row, 1, "Equipment", false);
                        spawn_tab_button(row, 2, "Lore", false);
                    });

                // Tab 0: Stats
                spawn_tab_panel(tab_group, 0, true, |panel| {
                    spawn_stats_tab(panel, &stats);
                });

                // Tab 1: Equipment
                spawn_tab_panel(tab_group, 1, false, |panel| {
                    spawn_equipment_tab(panel, &equip, &inv, &asset_server);
                });

                // Tab 2: Lore
                spawn_tab_panel(tab_group, 2, false, |panel| {
                    spawn_lore_tab(panel);
                });
            });
        });
}

// ============================================================
// Sync systems
// ============================================================

fn sync_bars(
    stats: Res<CharacterStats>,
    mut hp_bar: Query<&mut ProgressBar, (With<HealthBar>, Without<ManaBar>, Without<StaminaBar>)>,
    mut mp_bar: Query<&mut ProgressBar, (With<ManaBar>, Without<HealthBar>, Without<StaminaBar>)>,
    mut sp_bar: Query<&mut ProgressBar, (With<StaminaBar>, Without<HealthBar>, Without<ManaBar>)>,
    mut hp_text: Query<&mut Text, (With<HealthText>, Without<ManaText>, Without<StaminaText>)>,
    mut mp_text: Query<&mut Text, (With<ManaText>, Without<HealthText>, Without<StaminaText>)>,
    mut sp_text: Query<&mut Text, (With<StaminaText>, Without<HealthText>, Without<ManaText>)>,
) {
    if !stats.is_changed() {
        return;
    }

    if let Ok(mut bar) = hp_bar.single_mut() {
        bar.set(stats.health / stats.health_max);
    }
    if let Ok(mut bar) = mp_bar.single_mut() {
        bar.set(stats.mana / stats.mana_max);
    }
    if let Ok(mut bar) = sp_bar.single_mut() {
        bar.set(stats.stamina / stats.stamina_max);
    }
    if let Ok(mut text) = hp_text.single_mut() {
        **text = format!("{:.0}/{:.0}", stats.health, stats.health_max);
    }
    if let Ok(mut text) = mp_text.single_mut() {
        **text = format!("{:.0}/{:.0}", stats.mana, stats.mana_max);
    }
    if let Ok(mut text) = sp_text.single_mut() {
        **text = format!("{:.0}/{:.0}", stats.stamina, stats.stamina_max);
    }
}

const COLOR_IMPROVED: Color = Color::srgb(0.3, 0.9, 0.4);
const COLOR_WORSENED: Color = Color::srgb(0.9, 0.4, 0.3);
const FLASH_DURATION: f32 = 1.5;

fn sync_attributes(
    stats: Res<CharacterStats>,
    mut prev: ResMut<PreviousStats>,
    mut commands: Commands,
    mut values: Query<(Entity, &AttributeValue, &mut Text, &mut TextColor), Without<PointsText>>,
    mut bars: Query<(&AttributeBar, &mut ProgressBar)>,
    mut points: Query<
        &mut Text,
        (
            With<PointsText>,
            Without<AttributeValue>,
            Without<DerivedStatText>,
        ),
    >,
    mut derived: Query<
        (Entity, &DerivedStatText, &mut Text, &mut TextColor),
        (Without<PointsText>, Without<AttributeValue>),
    >,
) {
    if !stats.is_changed() {
        return;
    }

    for (entity, attr_val, mut text, mut color) in &mut values {
        let delta = prev.attr_delta(attr_val.0, &stats);
        **text = format!("{}", attr_val.0.get(&stats));
        if delta != 0 {
            let flash_color = if delta > 0 { COLOR_IMPROVED } else { COLOR_WORSENED };
            color.0 = flash_color;
            commands.entity(entity).insert(ChangeFlash {
                timer: FLASH_DURATION,
                color: flash_color,
            });
        }
    }

    for (attr_bar, mut bar) in &mut bars {
        bar.set(attr_bar.0.get(&stats) as f32 / 30.0);
    }

    if let Ok(mut text) = points.single_mut() {
        **text = format!("Available: {}", stats.available_points);
    }

    // Store old derived values for comparison
    let old_derived: Vec<(&str, String)> = derived
        .iter()
        .map(|(_, stat, text, _)| (stat.0, text.0.clone()))
        .collect();

    for (entity, stat, mut text, mut color) in &mut derived {
        let new_val = stats_tab::derived_value(stat.0, &stats);
        let old_val = old_derived.iter().find(|(k, _)| *k == stat.0);
        if let Some((_, old)) = old_val {
            if *old != new_val {
                // Parse numeric part for comparison
                let old_num: f32 = old.trim_end_matches('%').parse().unwrap_or(0.0);
                let new_num: f32 = new_val.trim_end_matches('%').parse().unwrap_or(0.0);
                let flash_color = if new_num > old_num { COLOR_IMPROVED } else { COLOR_WORSENED };
                color.0 = flash_color;
                commands.entity(entity).insert(ChangeFlash {
                    timer: FLASH_DURATION,
                    color: flash_color,
                });
            }
        }
        **text = new_val;
    }

    *prev = PreviousStats::snapshot(&stats);
}

fn fade_change_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChangeFlash, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    let base_color = Color::srgb(0.85, 0.85, 0.85);

    for (entity, mut flash, mut text_color) in &mut query {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            text_color.0 = base_color;
            commands.entity(entity).remove::<ChangeFlash>();
        } else {
            // Lerp from flash color back to base
            let t = (flash.timer / FLASH_DURATION).clamp(0.0, 1.0);
            let flash_srgba = flash.color.to_srgba();
            let base_srgba = base_color.to_srgba();
            text_color.0 = Color::srgb(
                base_srgba.red + (flash_srgba.red - base_srgba.red) * t,
                base_srgba.green + (flash_srgba.green - base_srgba.green) * t,
                base_srgba.blue + (flash_srgba.blue - base_srgba.blue) * t,
            );
        }
    }
}
