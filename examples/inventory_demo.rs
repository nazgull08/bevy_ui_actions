//! Minimal inventory example with actual item movement.
//!
//! Run: `cargo run --example inventory_demo -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<Inventory>()
        .add_systems(Startup, setup)
        .add_systems(Update, sync_visuals)
        .run();
}

// ============ Data ============

const SLOT_COUNT: usize = 6;
const EMPTY: Color = Color::srgb(0.2, 0.2, 0.2);
const RED: Color = Color::srgb(0.8, 0.3, 0.3);
const GREEN: Color = Color::srgb(0.3, 0.8, 0.3);
const BLUE: Color = Color::srgb(0.3, 0.3, 0.8);

#[derive(Resource)]
struct Inventory {
    slots: [Option<Color>; SLOT_COUNT],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [Some(RED), Some(GREEN), Some(BLUE), None, None, None],
        }
    }
}

#[derive(Component)]
struct Slot(usize);

// ============ Action ============

struct DropAction {
    target: usize,
}

impl UiAction for DropAction {
    fn execute(&self, world: &mut World) {
        // Where did we drag from?
        let source = {
            let drag_state = world.resource::<DragState>();
            drag_state
                .dragging
                .and_then(|e| world.get::<Slot>(e).map(|s| s.0))
        };

        let Some(source) = source else { return };

        if source == self.target {
            return;
        }

        // Swap
        let mut inv = world.resource_mut::<Inventory>();
        inv.slots.swap(source, self.target);

        info!("Moved: slot {} -> slot {}", source, self.target);
    }
}

// ============ Setup ============

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            for i in 0..SLOT_COUNT {
                parent.spawn((
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(EMPTY),
                    BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                    Slot(i),
                    Draggable,
                    DropTarget,
                    OnDrop::new(DropAction { target: i }),
                    Interaction::None,
                ));
            }
        });
}

// ============ Sync ============

fn sync_visuals(inventory: Res<Inventory>, mut query: Query<(&Slot, &mut BackgroundColor)>) {
    if !inventory.is_changed() {
        return;
    }

    for (slot, mut bg) in &mut query {
        *bg = BackgroundColor(inventory.slots[slot.0].unwrap_or(EMPTY));
    }
}
