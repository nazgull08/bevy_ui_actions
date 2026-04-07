use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

use crate::data::*;

const SLOT_SIZE: f32 = 72.0;
const INV_SLOT_SIZE: f32 = 64.0;
const ICON_PADDING: f32 = 4.0;
const SLOT_EMPTY_BG: Color = Color::srgb(0.15, 0.15, 0.18);
const SLOT_FILLED_BG: Color = Color::srgb(0.22, 0.22, 0.28);
const EQUIP_EMPTY_BG: Color = Color::srgb(0.12, 0.14, 0.18);
const EQUIP_FILLED_BG: Color = Color::srgb(0.18, 0.22, 0.28);
const SLOT_HIGHLIGHT_BORDER: Color = Color::srgb(0.5, 0.7, 0.3);
const SLOT_INCOMPATIBLE_BG: Color = Color::srgb(0.10, 0.10, 0.12);
const EQUIP_DEFAULT_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);
const INV_DEFAULT_BORDER: Color = Color::srgb(0.25, 0.25, 0.3);

/// Marker on the icon image child of a slot.
#[derive(Component)]
pub(crate) struct SlotIcon;

/// Marker on the name text child of a slot.
#[derive(Component)]
struct SlotName;

/// Marker on the label text child (shown when slot is empty).
#[derive(Component)]
struct SlotLabel;

pub fn spawn_equipment_tab(
    parent: &mut ChildSpawnerCommands,
    equip: &EquipmentState,
    inv: &InventoryState,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|root| {
            spawn_equipment_panel(root, equip, asset_server);
            spawn_inventory_panel(root, inv, equip, asset_server);
        });
}

fn spawn_equipment_panel(
    parent: &mut ChildSpawnerCommands,
    equip: &EquipmentState,
    asset_server: &AssetServer,
) {
    parent
        .spawn_panel(PanelConfig {
            width: Val::Px(200.0),
            height: Val::Auto,
            gap: 12.0,
            padding: 12.0,
            direction: FlexDirection::Column,
            ..PanelConfig::dark()
        })
        .with_children(|panel| {
            panel.ui_text(TextRole::Heading, "Equipment");

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|grid| {
                    spawn_equip_slot(grid, EquipSlot::Head, equip, asset_server);
                    spawn_equip_slot(grid, EquipSlot::Chest, equip, asset_server);

                    grid.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_equip_slot(row, EquipSlot::MainHand, equip, asset_server);
                        spawn_equip_slot(row, EquipSlot::OffHand, equip, asset_server);
                    });
                });
        });
}

fn spawn_equip_slot(
    parent: &mut ChildSpawnerCommands,
    slot: EquipSlot,
    equip: &EquipmentState,
    asset_server: &AssetServer,
) {
    let item = equip.get(slot);
    let has_item = item.is_some();
    let bg = if has_item { EQUIP_FILLED_BG } else { EQUIP_EMPTY_BG };

    let mut ec = parent
        .spawn((
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(EQUIP_DEFAULT_BORDER),
            DropTarget,
            OnDrop::new(DropToEquipSlot { target_slot: slot }),
            EquipSlotUI(slot),
            Interaction::None,
            OnRightClick::new(ShowUnequipModal { slot }),
        ));

    if let Some(idx) = item {
        ec.insert((Draggable, build_item_tooltip(idx)));
    }

    ec.with_children(|s| {
        let icon_size = SLOT_SIZE - ICON_PADDING * 2.0 - 2.0; // minus border
        // Icon image (always present, visibility toggled)
        let mut icon_ec = s.spawn((
            ImageNode::default(),
            Node {
                width: Val::Px(icon_size),
                height: Val::Px(icon_size),
                position_type: PositionType::Absolute,
                left: Val::Px(ICON_PADDING),
                top: Val::Px(ICON_PADDING),
                ..default()
            },
            SlotIcon,
            if has_item { Visibility::Visible } else { Visibility::Hidden },
        ));
        if let Some(idx) = item {
            icon_ec.insert(ImageNode::new(asset_server.load(ITEMS[idx].icon)));
        }

        // Name text (bottom overlay)
        let name_str = item.map(|idx| ITEMS[idx].name).unwrap_or("");
        s.spawn((
            Text::new(name_str),
            TextFont {
                font_size: 9.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.85)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(2.0),
                ..default()
            },
            SlotName,
            if has_item { Visibility::Visible } else { Visibility::Hidden },
        ));

        // Label (shown when empty)
        s.ui_text_styled(slot.label(), 11.0, Color::srgb(0.35, 0.35, 0.4))
            .insert((
                SlotLabel,
                if has_item { Visibility::Hidden } else { Visibility::Visible },
            ));
    });
}

fn spawn_inventory_panel(
    parent: &mut ChildSpawnerCommands,
    inv: &InventoryState,
    equip: &EquipmentState,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|col| {
            col.ui_text(TextRole::Heading, "Inventory");

            col.spawn_scroll_view(ScrollViewConfig {
                height: Val::Percent(100.0),
                background: Some(Color::srgb(0.08, 0.08, 0.10)),
                ..default()
            })
            .with_children(|scroll| {
                scroll
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(6.0),
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|grid| {
                        for (i, slot) in inv.items.iter().enumerate() {
                            spawn_inv_slot(grid, i, *slot, equip, asset_server);
                        }
                    });
            });

            col.ui_text_styled(
                "Drag to equip \u{2022} Right-click to unequip \u{2022} Hover for comparison",
                12.0,
                Color::srgb(0.4, 0.4, 0.45),
            );
        });
}

fn spawn_inv_slot(
    parent: &mut ChildSpawnerCommands,
    inv_idx: usize,
    item: Option<usize>,
    equip: &EquipmentState,
    asset_server: &AssetServer,
) {
    let has_item = item.is_some();
    let bg = if has_item { SLOT_FILLED_BG } else { SLOT_EMPTY_BG };

    let mut ec = parent.spawn((
        Node {
            width: Val::Px(INV_SLOT_SIZE),
            height: Val::Px(INV_SLOT_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(bg),
        BorderColor(INV_DEFAULT_BORDER),
        DropTarget,
        OnDrop::new(DropToInvSlot { target_idx: inv_idx }),
        InvSlot(inv_idx),
        Interaction::None,
    ));

    if let Some(idx) = item {
        ec.insert((Draggable, build_inv_item_tooltip(idx, equip)));
    }

    ec.with_children(|s| {
        let icon_size = INV_SLOT_SIZE - ICON_PADDING * 2.0 - 2.0;
        // Icon image
        let mut icon_ec = s.spawn((
            ImageNode::default(),
            Node {
                width: Val::Px(icon_size),
                height: Val::Px(icon_size),
                position_type: PositionType::Absolute,
                left: Val::Px(ICON_PADDING),
                top: Val::Px(ICON_PADDING),
                ..default()
            },
            SlotIcon,
            if has_item { Visibility::Visible } else { Visibility::Hidden },
        ));
        if let Some(idx) = item {
            icon_ec.insert(ImageNode::new(asset_server.load(ITEMS[idx].icon)));
        }

        // Name text
        let name_str = item.map(|idx| ITEMS[idx].name).unwrap_or("");
        s.spawn((
            Text::new(name_str),
            TextFont {
                font_size: 9.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.75, 0.75)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(1.0),
                ..default()
            },
            SlotName,
            if has_item { Visibility::Visible } else { Visibility::Hidden },
        ));
    });
}

// -- Tooltip helper --

fn build_item_tooltip(item_idx: usize) -> Tooltip {
    let def = &ITEMS[item_idx];
    let mut b = Tooltip::builder()
        .title(def.name)
        .subtitle(def.slot.label())
        .separator();

    if def.str_bonus != 0 {
        b = b.stat("Strength", format!("{:+}", def.str_bonus));
    }
    if def.dex_bonus != 0 {
        b = b.stat("Dexterity", format!("{:+}", def.dex_bonus));
    }
    if def.int_bonus != 0 {
        b = b.stat("Intelligence", format!("{:+}", def.int_bonus));
    }
    if def.vit_bonus != 0 {
        b = b.stat("Vitality", format!("{:+}", def.vit_bonus));
    }

    if def.has_bonuses() {
        b = b.separator();
    }

    b.text(def.description).delay(300).build()
}

/// Build tooltip for an inventory item, comparing against currently equipped item in that slot.
fn build_inv_item_tooltip(item_idx: usize, equip: &EquipmentState) -> Tooltip {
    let def = &ITEMS[item_idx];
    let equipped_idx = equip.get(def.slot);

    let mut b = Tooltip::builder()
        .title(def.name)
        .subtitle(def.slot.label())
        .separator();

    // Show bonuses with diff against currently equipped item
    let (eq_str, eq_dex, eq_int, eq_vit) = if let Some(eq_idx) = equipped_idx {
        let eq = &ITEMS[eq_idx];
        (eq.str_bonus, eq.dex_bonus, eq.int_bonus, eq.vit_bonus)
    } else {
        (0, 0, 0, 0)
    };

    b = add_bonus_stat_diff(b, "Strength", def.str_bonus, eq_str);
    b = add_bonus_stat_diff(b, "Dexterity", def.dex_bonus, eq_dex);
    b = add_bonus_stat_diff(b, "Intelligence", def.int_bonus, eq_int);
    b = add_bonus_stat_diff(b, "Vitality", def.vit_bonus, eq_vit);

    if def.has_bonuses() {
        b = b.separator();
    }

    b.text(def.description).delay(300).build()
}

fn add_bonus_stat_diff(
    b: TooltipBuilder,
    label: &str,
    item_bonus: i8,
    equipped_bonus: i8,
) -> TooltipBuilder {
    if item_bonus == 0 && equipped_bonus == 0 {
        return b;
    }
    let delta = item_bonus - equipped_bonus;
    let diff = if delta > 0 {
        StatDiff::Better(delta as f32)
    } else if delta < 0 {
        StatDiff::Worse(delta.unsigned_abs() as f32)
    } else {
        StatDiff::Neutral
    };
    b.stat_diff(label, format!("{:+}", item_bonus), diff)
}

// -- Sync system --

pub fn sync_equipment(
    equip: Res<EquipmentState>,
    inv: Res<InventoryState>,
    asset_server: Res<AssetServer>,
    equip_slots: Query<(Entity, &EquipSlotUI, &Children), Without<InvSlot>>,
    inv_slots: Query<(Entity, &InvSlot, &Children), Without<EquipSlotUI>>,
    mut bg_query: Query<&mut BackgroundColor>,
    mut image_query: Query<&mut ImageNode, With<SlotIcon>>,
    mut text_query: Query<&mut Text>,
    mut vis_query: Query<&mut Visibility>,
    mut commands: Commands,
) {
    if !equip.is_changed() && !inv.is_changed() {
        return;
    }

    // Sync equipment slots
    for (entity, slot_ui, children) in &equip_slots {
        let item = equip.get(slot_ui.0);
        let has_item = item.is_some();

        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = BackgroundColor(if has_item { EQUIP_FILLED_BG } else { EQUIP_EMPTY_BG });
        }

        if let Some(idx) = item {
            commands.entity(entity).insert((Draggable, build_item_tooltip(idx)));
        } else {
            commands.entity(entity).remove::<(Draggable, Tooltip)>();
        }

        sync_slot_children(
            children,
            item,
            &asset_server,
            &mut image_query,
            &mut text_query,
            &mut vis_query,
        );
    }

    // Sync inventory slots
    for (entity, inv_slot, children) in &inv_slots {
        let item = inv.items.get(inv_slot.0).copied().flatten();
        let has_item = item.is_some();

        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = BackgroundColor(if has_item { SLOT_FILLED_BG } else { SLOT_EMPTY_BG });
        }

        if let Some(idx) = item {
            commands
                .entity(entity)
                .insert((Draggable, build_inv_item_tooltip(idx, &equip)));
        } else {
            commands.entity(entity).remove::<(Draggable, Tooltip)>();
        }

        sync_slot_children(
            children,
            item,
            &asset_server,
            &mut image_query,
            &mut text_query,
            &mut vis_query,
        );
    }
}

fn sync_slot_children(
    children: &Children,
    item: Option<usize>,
    asset_server: &AssetServer,
    image_query: &mut Query<&mut ImageNode, With<SlotIcon>>,
    text_query: &mut Query<&mut Text>,
    vis_query: &mut Query<&mut Visibility>,
) {
    let has_item = item.is_some();
    let child_list: Vec<Entity> = children.iter().collect();

    // Icon image (index 0)
    if let Some(&icon_entity) = child_list.first() {
        if let Ok(mut image) = image_query.get_mut(icon_entity) {
            if let Some(idx) = item {
                image.image = asset_server.load(ITEMS[idx].icon);
            }
        }
        if let Ok(mut vis) = vis_query.get_mut(icon_entity) {
            *vis = if has_item { Visibility::Visible } else { Visibility::Hidden };
        }
    }

    // Name text (index 1)
    if let Some(&name_entity) = child_list.get(1) {
        if let Ok(mut text) = text_query.get_mut(name_entity) {
            **text = item.map(|idx| ITEMS[idx].name.to_string()).unwrap_or_default();
        }
        if let Ok(mut vis) = vis_query.get_mut(name_entity) {
            *vis = if has_item { Visibility::Visible } else { Visibility::Hidden };
        }
    }

    // Label (index 2, only equip slots have it)
    if let Some(&label_entity) = child_list.get(2) {
        if let Ok(mut vis) = vis_query.get_mut(label_entity) {
            *vis = if has_item { Visibility::Hidden } else { Visibility::Visible };
        }
    }
}

// -- Drag highlight system --

pub fn highlight_compatible_slots(
    drag: Res<DragState>,
    inv: Res<InventoryState>,
    equip: Res<EquipmentState>,
    mut equip_slots: Query<
        (Entity, &EquipSlotUI, &mut BorderColor, &mut BackgroundColor),
        Without<InvSlot>,
    >,
    mut inv_slots: Query<
        (Entity, &InvSlot, &mut BorderColor, &mut BackgroundColor),
        Without<EquipSlotUI>,
    >,
) {
    // Resolve dragged item index from entity
    let item_idx: Option<usize> = drag.dragging.and_then(|source| {
        // Check if source is an inv slot
        for (entity, inv_slot, _, _) in inv_slots.iter() {
            if entity == source {
                return inv.items.get(inv_slot.0).copied().flatten();
            }
        }
        // Check if source is an equip slot
        for (entity, equip_slot, _, _) in equip_slots.iter() {
            if entity == source {
                return equip.get(equip_slot.0);
            }
        }
        None
    });

    // Equipment slots
    for (entity, slot_ui, mut border, mut bg) in &mut equip_slots {
        let has_item = equip.get(slot_ui.0).is_some();
        match item_idx {
            Some(idx) if drag.dragging != Some(entity) => {
                let compatible = ITEMS[idx].slot == slot_ui.0;
                if compatible {
                    border.0 = SLOT_HIGHLIGHT_BORDER;
                } else {
                    border.0 = EQUIP_DEFAULT_BORDER;
                    bg.0 = SLOT_INCOMPATIBLE_BG;
                    continue;
                }
            }
            _ => {
                border.0 = EQUIP_DEFAULT_BORDER;
            }
        }
        bg.0 = if has_item { EQUIP_FILLED_BG } else { EQUIP_EMPTY_BG };
    }

    // Inventory slots — always valid drop targets
    for (entity, inv_slot, mut border, mut bg) in &mut inv_slots {
        let has_item = inv.items.get(inv_slot.0).copied().flatten().is_some();
        match item_idx {
            Some(_) if drag.dragging != Some(entity) => {
                border.0 = SLOT_HIGHLIGHT_BORDER;
            }
            _ => {
                border.0 = INV_DEFAULT_BORDER;
            }
        }
        bg.0 = if has_item { SLOT_FILLED_BG } else { SLOT_EMPTY_BG };
    }
}
