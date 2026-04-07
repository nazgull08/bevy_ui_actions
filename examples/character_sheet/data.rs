use bevy::prelude::*;
use bevy_ui_actions::prelude::{DragState, ModalQueue, ModalRequest, UiAction};

#[derive(Resource)]
pub struct CharacterStats {
    pub health: f32,
    pub health_max: f32,
    pub mana: f32,
    pub mana_max: f32,
    pub stamina: f32,
    pub stamina_max: f32,
    pub strength: u8,
    pub dexterity: u8,
    pub intelligence: u8,
    pub vitality: u8,
    pub available_points: u8,
    pub xp: f32,
    pub xp_max: f32,
    pub level: u8,
    // Base values (minimum when decrementing)
    pub base_strength: u8,
    pub base_dexterity: u8,
    pub base_intelligence: u8,
    pub base_vitality: u8,
}

impl Default for CharacterStats {
    fn default() -> Self {
        let mut s = Self {
            health: 0.0,
            health_max: 0.0,
            mana: 0.0,
            mana_max: 0.0,
            stamina: 0.0,
            stamina_max: 0.0,
            strength: 10,
            dexterity: 8,
            intelligence: 12,
            vitality: 9,
            available_points: 3,
            xp: 1250.0,
            xp_max: 2000.0,
            level: 5,
            base_strength: 10,
            base_dexterity: 8,
            base_intelligence: 12,
            base_vitality: 9,
        };
        s.recalculate();
        // Start slightly damaged to show bars aren't full
        s.health = s.health_max * 0.75;
        s.mana = s.mana_max * 0.5;
        s.stamina = s.stamina_max * 0.9;
        s
    }
}

impl CharacterStats {
    /// Recalculate derived max values from attributes.
    /// Vitality → health_max, Intelligence → mana_max, Dexterity → stamina_max.
    pub fn recalculate(&mut self) {
        self.health_max = 50.0 + self.vitality as f32 * 10.0;
        self.mana_max = 20.0 + self.intelligence as f32 * 5.0;
        self.stamina_max = 60.0 + self.dexterity as f32 * 5.0;

        // Clamp current values to new max (don't inflate)
        self.health = self.health.min(self.health_max);
        self.mana = self.mana.min(self.mana_max);
        self.stamina = self.stamina.min(self.stamina_max);
    }

    pub fn base_value(&self, attr: Attribute) -> u8 {
        match attr {
            Attribute::Strength => self.base_strength,
            Attribute::Dexterity => self.base_dexterity,
            Attribute::Intelligence => self.base_intelligence,
            Attribute::Vitality => self.base_vitality,
        }
    }
}

// -- Markers --

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct ManaBar;

#[derive(Component)]
pub struct StaminaBar;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct ManaText;

#[derive(Component)]
pub struct StaminaText;

#[derive(Component)]
pub struct XpBar;

#[derive(Component)]
pub struct XpText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct PointsText;

#[derive(Component, Clone, Copy)]
pub enum Attribute {
    Strength,
    Dexterity,
    Intelligence,
    Vitality,
}

#[derive(Component)]
pub struct AttributeValue(pub Attribute);

#[derive(Component)]
pub struct AttributeBar(pub Attribute);

#[derive(Component)]
pub struct DerivedStatText(pub &'static str);

// -- Change highlight --

#[derive(Resource, Default)]
pub struct PreviousStats {
    pub strength: u8,
    pub dexterity: u8,
    pub intelligence: u8,
    pub vitality: u8,
}

impl PreviousStats {
    pub fn snapshot(stats: &CharacterStats) -> Self {
        Self {
            strength: stats.strength,
            dexterity: stats.dexterity,
            intelligence: stats.intelligence,
            vitality: stats.vitality,
        }
    }

    pub fn attr_delta(&self, attr: Attribute, stats: &CharacterStats) -> i8 {
        let (old, new) = match attr {
            Attribute::Strength => (self.strength, stats.strength),
            Attribute::Dexterity => (self.dexterity, stats.dexterity),
            Attribute::Intelligence => (self.intelligence, stats.intelligence),
            Attribute::Vitality => (self.vitality, stats.vitality),
        };
        new as i8 - old as i8
    }
}

/// Marker: this text should flash on change.
#[derive(Component)]
pub struct ChangeFlash {
    pub timer: f32,
    pub color: Color,
}

// -- Equipment --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Head,
    Chest,
    MainHand,
    OffHand,
}

impl EquipSlot {
    pub const ALL: [EquipSlot; 4] = [
        EquipSlot::Head,
        EquipSlot::Chest,
        EquipSlot::MainHand,
        EquipSlot::OffHand,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Chest => "Chest",
            Self::MainHand => "Main Hand",
            Self::OffHand => "Off Hand",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Head => 0,
            Self::Chest => 1,
            Self::MainHand => 2,
            Self::OffHand => 3,
        }
    }
}

pub struct ItemDef {
    pub name: &'static str,
    pub slot: EquipSlot,
    pub description: &'static str,
    pub icon: &'static str,
    /// GLB model path (relative to assets/), or empty for procedural visuals.
    pub model: &'static str,
    pub str_bonus: i8,
    pub dex_bonus: i8,
    pub int_bonus: i8,
    pub vit_bonus: i8,
}

impl ItemDef {
    pub fn has_bonuses(&self) -> bool {
        self.str_bonus != 0 || self.dex_bonus != 0 || self.int_bonus != 0 || self.vit_bonus != 0
    }
}

pub const ITEMS: &[ItemDef] = &[
    // -- Head --
    ItemDef {
        name: "Iron Helm",
        slot: EquipSlot::Head,
        description: "A sturdy iron helmet.",
        icon: "icons/iron_helm.png",
        model: "models/iron_helm.glb",
        str_bonus: 0,
        dex_bonus: 0,
        int_bonus: 0,
        vit_bonus: 2,
    },
    ItemDef {
        name: "Wizard Hat",
        slot: EquipSlot::Head,
        description: "A pointed hat crackling with arcane energy.",
        icon: "icons/wizard_hat.png",
        model: "models/wizard_hat.glb",
        str_bonus: 0,
        dex_bonus: 0,
        int_bonus: 3,
        vit_bonus: -1,
    },
    // -- Chest --
    ItemDef {
        name: "Chain Mail",
        slot: EquipSlot::Chest,
        description: "Interlocking iron rings.",
        icon: "icons/chain_mail.png",
        model: "models/chain_mail.glb",
        str_bonus: 0,
        dex_bonus: -1,
        int_bonus: 0,
        vit_bonus: 3,
    },
    ItemDef {
        name: "Leather Vest",
        slot: EquipSlot::Chest,
        description: "Light and supple, favored by scouts.",
        icon: "icons/leather_vest.png",
        model: "models/lether_vest.glb",
        str_bonus: 0,
        dex_bonus: 2,
        int_bonus: 0,
        vit_bonus: 1,
    },
    // -- MainHand --
    ItemDef {
        name: "Iron Sword",
        slot: EquipSlot::MainHand,
        description: "A reliable blade.",
        icon: "icons/iron_sword.png",
        model: "",
        str_bonus: 2,
        dex_bonus: 1,
        int_bonus: 0,
        vit_bonus: 0,
    },
    ItemDef {
        name: "Battle Axe",
        slot: EquipSlot::MainHand,
        description: "Heavy and devastating. Not for the nimble.",
        icon: "icons/battle_axe.png",
        model: "",
        str_bonus: 4,
        dex_bonus: -2,
        int_bonus: 0,
        vit_bonus: 0,
    },
    ItemDef {
        name: "Arcane Staff",
        slot: EquipSlot::MainHand,
        description: "Channels raw magical force.",
        icon: "icons/arcane_staff.png",
        model: "",
        str_bonus: -1,
        dex_bonus: 0,
        int_bonus: 3,
        vit_bonus: 0,
    },
    // -- OffHand --
    ItemDef {
        name: "Wooden Shield",
        slot: EquipSlot::OffHand,
        description: "A battered wooden shield.",
        icon: "icons/wooden_shield.png",
        model: "",
        str_bonus: 1,
        dex_bonus: 0,
        int_bonus: 0,
        vit_bonus: 1,
    },
    ItemDef {
        name: "Tome of Lore",
        slot: EquipSlot::OffHand,
        description: "Ancient knowledge bound in leather.",
        icon: "icons/tome_of_lore.png",
        model: "",
        str_bonus: 0,
        dex_bonus: 0,
        int_bonus: 2,
        vit_bonus: 0,
    },
];

pub fn apply_item_bonuses(stats: &mut CharacterStats, item_idx: usize) {
    let def = &ITEMS[item_idx];
    stats.strength = (stats.strength as i8 + def.str_bonus).max(1) as u8;
    stats.dexterity = (stats.dexterity as i8 + def.dex_bonus).max(1) as u8;
    stats.intelligence = (stats.intelligence as i8 + def.int_bonus).max(1) as u8;
    stats.vitality = (stats.vitality as i8 + def.vit_bonus).max(1) as u8;
    stats.recalculate();
}

pub fn remove_item_bonuses(stats: &mut CharacterStats, item_idx: usize) {
    let def = &ITEMS[item_idx];
    stats.strength = (stats.strength as i8 - def.str_bonus).max(1) as u8;
    stats.dexterity = (stats.dexterity as i8 - def.dex_bonus).max(1) as u8;
    stats.intelligence = (stats.intelligence as i8 - def.int_bonus).max(1) as u8;
    stats.vitality = (stats.vitality as i8 - def.vit_bonus).max(1) as u8;
    stats.recalculate();
}

/// Which item is in each equipment slot.
#[derive(Resource, Default)]
pub struct EquipmentState {
    pub slots: [Option<usize>; 4],
}

impl EquipmentState {
    pub fn get(&self, slot: EquipSlot) -> Option<usize> {
        self.slots[slot.index()]
    }

    pub fn set(&mut self, slot: EquipSlot, item: Option<usize>) {
        self.slots[slot.index()] = item;
    }
}

/// Items available in inventory (indices into ITEMS).
#[derive(Resource)]
pub struct InventoryState {
    pub items: Vec<Option<usize>>,
}

impl Default for InventoryState {
    fn default() -> Self {
        let mut items: Vec<Option<usize>> = (0..ITEMS.len()).map(Some).collect();
        items.resize(16, None);
        Self { items }
    }
}

impl InventoryState {
    pub fn add(&mut self, item_idx: usize) {
        // Put in first empty slot, or append
        if let Some(slot) = self.items.iter_mut().find(|s| s.is_none()) {
            *slot = Some(item_idx);
        } else {
            self.items.push(Some(item_idx));
        }
    }

    /// Remove item from inventory, returning the slot index it was in.
    pub fn remove(&mut self, item_idx: usize) -> Option<usize> {
        if let Some(pos) = self.items.iter().position(|s| *s == Some(item_idx)) {
            self.items[pos] = None;
            Some(pos)
        } else {
            None
        }
    }
}

/// Marker on inventory slot UI entities.
#[derive(Component)]
pub struct InvSlot(pub usize);

/// Marker on equipment slot UI entities.
#[derive(Component)]
pub struct EquipSlotUI(pub EquipSlot);

// -- Equipment Actions --

/// Drop onto an equipment slot. Source can be inventory or another equip slot.
pub struct DropToEquipSlot {
    pub target_slot: EquipSlot,
}

impl UiAction for DropToEquipSlot {
    fn execute(&self, world: &mut World) {
        let dragging = world.resource::<DragState>().dragging;
        let Some(source) = dragging else { return };

        // Source is inventory slot?
        if let Some(&InvSlot(inv_idx)) = world.get::<InvSlot>(source) {
            let item_idx = {
                let inv = world.resource::<InventoryState>();
                inv.items.get(inv_idx).copied().flatten()
            };
            let Some(item_idx) = item_idx else { return };

            // Check slot compatibility
            if ITEMS[item_idx].slot != self.target_slot {
                return;
            }

            let old_item = world.resource::<EquipmentState>().get(self.target_slot);

            // Remove old item bonuses, apply new
            if let Some(old) = old_item {
                remove_item_bonuses(&mut world.resource_mut::<CharacterStats>(), old);
            }
            apply_item_bonuses(&mut world.resource_mut::<CharacterStats>(), item_idx);

            world.resource_mut::<EquipmentState>().set(self.target_slot, Some(item_idx));

            let mut inv = world.resource_mut::<InventoryState>();
            let removed_slot = inv.remove(item_idx);
            if let Some(old) = old_item {
                // Place old item into the same inv slot the new item came from
                if let Some(slot_idx) = removed_slot {
                    inv.items[slot_idx] = Some(old);
                } else {
                    inv.add(old);
                }
            }
            return;
        }

        // Source is another equip slot?
        if let Some(&EquipSlotUI(src_slot)) = world.get::<EquipSlotUI>(source) {
            if src_slot == self.target_slot {
                return;
            }
            let src_item = world.resource::<EquipmentState>().get(src_slot);
            let Some(item_idx) = src_item else { return };

            // Check slot compatibility
            if ITEMS[item_idx].slot != self.target_slot {
                return;
            }

            let mut equip = world.resource_mut::<EquipmentState>();
            let old_item = equip.get(self.target_slot);
            equip.set(self.target_slot, Some(item_idx));
            equip.set(src_slot, old_item);
            // Swapping between equip slots — net bonuses unchanged
        }
    }
}

/// Drop onto an inventory slot. Source can be an equip slot.
pub struct DropToInvSlot {
    pub target_idx: usize,
}

impl UiAction for DropToInvSlot {
    fn execute(&self, world: &mut World) {
        let dragging = world.resource::<DragState>().dragging;
        let Some(source) = dragging else { return };

        // Source is equip slot → unequip to this inv slot
        if let Some(&EquipSlotUI(src_slot)) = world.get::<EquipSlotUI>(source) {
            let item = world.resource::<EquipmentState>().get(src_slot);
            let Some(item_idx) = item else { return };

            remove_item_bonuses(&mut world.resource_mut::<CharacterStats>(), item_idx);
            world.resource_mut::<EquipmentState>().set(src_slot, None);

            let mut inv = world.resource_mut::<InventoryState>();
            // Put into specific target slot if empty, otherwise first empty
            if inv.items.get(self.target_idx).copied().flatten().is_none() {
                inv.items[self.target_idx] = Some(item_idx);
            } else {
                inv.add(item_idx);
            }
            return;
        }

        // Source is another inv slot → swap
        if let Some(&InvSlot(src_idx)) = world.get::<InvSlot>(source) {
            if src_idx == self.target_idx {
                return;
            }
            let mut inv = world.resource_mut::<InventoryState>();
            inv.items.swap(src_idx, self.target_idx);
        }
    }
}

// -- Unequip Actions (right-click modal) --

pub struct ShowUnequipModal {
    pub slot: EquipSlot,
}

impl UiAction for ShowUnequipModal {
    fn execute(&self, world: &mut World) {
        let item = world.resource::<EquipmentState>().get(self.slot);
        let Some(item_idx) = item else { return };
        let item_name = ITEMS[item_idx].name.to_string();
        let slot_label = self.slot.label().to_string();
        let slot = self.slot;

        let mut queue = world.resource_mut::<ModalQueue>();
        queue.show(
            ModalRequest::confirm(
                format!("Unequip {}?", item_name),
                format!("Remove {} from {} slot?", item_name, slot_label),
            )
            .with_confirm(UnequipItem { slot }),
        );
    }
}

pub struct UnequipItem {
    pub slot: EquipSlot,
}

impl UiAction for UnequipItem {
    fn execute(&self, world: &mut World) {
        let item = world.resource::<EquipmentState>().get(self.slot);
        let Some(item_idx) = item else { return };

        remove_item_bonuses(&mut world.resource_mut::<CharacterStats>(), item_idx);
        world.resource_mut::<EquipmentState>().set(self.slot, None);
        world.resource_mut::<InventoryState>().add(item_idx);
    }
}

// -- Stat Actions --

pub struct IncrementAttribute(pub Attribute);

impl UiAction for IncrementAttribute {
    fn execute(&self, world: &mut World) {
        let mut stats = world.resource_mut::<CharacterStats>();
        if stats.available_points == 0 {
            return;
        }
        let val = match self.0 {
            Attribute::Strength => &mut stats.strength,
            Attribute::Dexterity => &mut stats.dexterity,
            Attribute::Intelligence => &mut stats.intelligence,
            Attribute::Vitality => &mut stats.vitality,
        };
        if *val < 30 {
            *val += 1;
            stats.available_points -= 1;
            stats.recalculate();
        }
    }
}

pub struct DecrementAttribute(pub Attribute);

impl UiAction for DecrementAttribute {
    fn execute(&self, world: &mut World) {
        let mut stats = world.resource_mut::<CharacterStats>();
        let base = stats.base_value(self.0);
        let val = match self.0 {
            Attribute::Strength => &mut stats.strength,
            Attribute::Dexterity => &mut stats.dexterity,
            Attribute::Intelligence => &mut stats.intelligence,
            Attribute::Vitality => &mut stats.vitality,
        };
        if *val > base {
            *val -= 1;
            stats.available_points += 1;
            stats.recalculate();
        }
    }
}

impl Attribute {
    pub fn get(self, stats: &CharacterStats) -> u8 {
        match self {
            Self::Strength => stats.strength,
            Self::Dexterity => stats.dexterity,
            Self::Intelligence => stats.intelligence,
            Self::Vitality => stats.vitality,
        }
    }

    pub fn bar_color(self) -> Color {
        match self {
            Self::Strength => Color::srgb(0.8, 0.35, 0.25),
            Self::Dexterity => Color::srgb(0.3, 0.75, 0.35),
            Self::Intelligence => Color::srgb(0.3, 0.4, 0.85),
            Self::Vitality => Color::srgb(0.75, 0.55, 0.2),
        }
    }

}

// -- Respec Action --

pub struct RespecAttributes;

impl UiAction for RespecAttributes {
    fn execute(&self, world: &mut World) {
        // Remove all equipment bonuses first
        let equipped: Vec<usize> = {
            let equip = world.resource::<EquipmentState>();
            EquipSlot::ALL.iter().filter_map(|s| equip.get(*s)).collect()
        };
        for idx in &equipped {
            remove_item_bonuses(&mut world.resource_mut::<CharacterStats>(), *idx);
        }

        // Reset attributes to base, reclaim spent points
        let mut stats = world.resource_mut::<CharacterStats>();
        let spent = (stats.strength - stats.base_strength)
            + (stats.dexterity - stats.base_dexterity)
            + (stats.intelligence - stats.base_intelligence)
            + (stats.vitality - stats.base_vitality);
        stats.strength = stats.base_strength;
        stats.dexterity = stats.base_dexterity;
        stats.intelligence = stats.base_intelligence;
        stats.vitality = stats.base_vitality;
        stats.available_points += spent;
        stats.recalculate();

        // Re-apply equipment bonuses
        for idx in &equipped {
            apply_item_bonuses(&mut world.resource_mut::<CharacterStats>(), *idx);
        }
    }
}

pub struct ShowRespecModal;

impl UiAction for ShowRespecModal {
    fn execute(&self, world: &mut World) {
        let mut queue = world.resource_mut::<ModalQueue>();
        queue.show(
            ModalRequest::confirm(
                "Reset Attributes",
                "Return all spent attribute points?\nEquipment bonuses will be preserved.",
            )
            .with_confirm(RespecAttributes),
        );
    }
}
