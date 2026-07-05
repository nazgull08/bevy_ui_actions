//! Container transfer demo — two inventories as floating windows, with drag&drop
//! item transfer between them. Built on the `spawn_window` + `spawn_slot_grid` widgets.
//!
//! Drag items between the two windows (transfer) or within a window (swap). Windows
//! are modeless, so cross-window drag works with no input scope. The chest window
//! (open by default) is toggled with `E`; items live on persistent entities.
//!
//! Demonstrates:
//! - Two inventories as separate entities (`DemoInventory`), windows are views.
//! - The library `Slot { container, index }` + `spawn_slot_grid` + `dragged_slot`:
//!   the widget wires structure/behavior, the caller owns data and the drop action.
//!
//! Run: `cargo run --example container_transfer -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (sync_slots, toggle_chest_window))
        .run();
}

// ============ Data ============

#[derive(Clone, Copy)]
struct DemoItem {
    #[allow(dead_code)] // used from Step 3 (labels / tooltips)
    name: &'static str,
    color: Color,
}

const ITEMS: &[DemoItem] = &[
    DemoItem {
        name: "Sword",
        color: Color::srgb(0.80, 0.30, 0.30),
    },
    DemoItem {
        name: "Potion",
        color: Color::srgb(0.30, 0.80, 0.30),
    },
    DemoItem {
        name: "Shield",
        color: Color::srgb(0.30, 0.50, 0.90),
    },
    DemoItem {
        name: "Gem",
        color: Color::srgb(0.85, 0.70, 0.20),
    },
    DemoItem {
        name: "Key",
        color: Color::srgb(0.70, 0.70, 0.70),
    },
];

const SLOT_COUNT: usize = 8;
/// Empty-slot color; matches `SlotGridConfig::default().background`.
const EMPTY_SLOT: Color = Color::srgb(0.18, 0.18, 0.20);

/// An inventory stored on a container entity. Each slot holds an index into [`ITEMS`].
#[derive(Component)]
struct DemoInventory {
    slots: Vec<Option<usize>>,
}

/// The persistent chest container entity (its window is spawned/despawned on `E`).
#[derive(Resource)]
struct ChestContainer(Entity);

/// Marker on the chest window root, so the toggle can find it.
#[derive(Component)]
struct ChestWindow;

// ============ Drag & drop ============

/// Drop action carried by every slot: move the dragged item here.
///
/// Same container → swap two slots. Different containers → transfer (and swap if
/// the destination is occupied). The source slot is read from `DragState`.
struct DropToSlot {
    container: Entity,
    index: usize,
}

impl UiAction for DropToSlot {
    fn execute(&self, world: &mut World) {
        let Some(source) = dragged_slot(world) else {
            return;
        };

        if source.container == self.container {
            if source.index == self.index {
                return;
            }
            if let Some(mut inv) = world.get_mut::<DemoInventory>(source.container) {
                inv.slots.swap(source.index, self.index);
            }
            return;
        }

        // Cross-container: read both items (Option<usize>), then write them swapped.
        let src_item = world
            .get::<DemoInventory>(source.container)
            .and_then(|inv| inv.slots[source.index]);
        let dst_item = world
            .get::<DemoInventory>(self.container)
            .and_then(|inv| inv.slots[self.index]);

        if let Some(mut inv) = world.get_mut::<DemoInventory>(source.container) {
            inv.slots[source.index] = dst_item;
        }
        if let Some(mut inv) = world.get_mut::<DemoInventory>(self.container) {
            inv.slots[self.index] = src_item;
        }
    }
}

// ============ Setup ============

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Persistent character container with some starting items.
    let character = commands
        .spawn(DemoInventory {
            slots: vec![Some(0), Some(1), None, Some(2), None, None, Some(3), None],
        })
        .id();

    // Character window — a view onto the character container.
    commands.spawn_window(
        WindowConfig::new("Character", Vec2::new(120.0, 120.0)).closable(false),
        |c| {
            c.spawn_slot_grid(
                &SlotGridConfig::default(),
                character,
                SLOT_COUNT,
                move |index| DropToSlot {
                    container: character,
                    index,
                },
                |_slot, _index| {},
            );
        },
    );

    // Persistent chest container, open by default. Its window is toggled with `E`.
    let chest = commands
        .spawn(DemoInventory {
            slots: vec![Some(4), None, Some(2), None, None, Some(0), None, None],
        })
        .id();
    commands.insert_resource(ChestContainer(chest));
    spawn_chest_window(&mut commands, chest);
}

/// Spawn the chest window (a view onto the chest container) and tag it.
fn spawn_chest_window(commands: &mut Commands, chest: Entity) {
    let window = commands.spawn_window(WindowConfig::new("Chest", Vec2::new(460.0, 120.0)), |c| {
        c.spawn_slot_grid(
            &SlotGridConfig::default(),
            chest,
            SLOT_COUNT,
            move |index| DropToSlot {
                container: chest,
                index,
            },
            |_slot, _index| {},
        );
    });
    commands.entity(window).insert(ChestWindow);
}

/// Open or close the chest window on `E`. Data lives on the chest entity, so the
/// window can be despawned and respawned freely.
fn toggle_chest_window(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    chest_window: Query<Entity, With<ChestWindow>>,
    container: Res<ChestContainer>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    if let Some(window) = chest_window.iter().next() {
        commands.entity(window).despawn();
    } else {
        spawn_chest_window(&mut commands, container.0);
    }
}

// ============ Sync ============

/// Paint each slot from its container's inventory and toggle `Draggable`
/// (only filled slots can be picked up).
fn sync_slots(
    inventories: Query<&DemoInventory>,
    mut slots: Query<(Entity, &Slot, &mut BackgroundColor, Has<Draggable>)>,
    mut commands: Commands,
) {
    for (entity, slot, mut bg, has_drag) in &mut slots {
        let item = inventories
            .get(slot.container)
            .ok()
            .and_then(|inv| inv.slots.get(slot.index).copied().flatten());

        *bg = BackgroundColor(item.map(|i| ITEMS[i].color).unwrap_or(EMPTY_SLOT));

        match (item.is_some(), has_drag) {
            (true, false) => {
                commands.entity(entity).insert(Draggable);
            }
            (false, true) => {
                commands.entity(entity).remove::<Draggable>();
            }
            _ => {}
        }
    }
}
