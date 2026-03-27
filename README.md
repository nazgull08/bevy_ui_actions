# bevy_ui_actions

Lightweight action-driven UI toolkit for [Bevy](https://bevyengine.org/).

Instead of matching `Interaction` changes in every system, define **action structs** that execute with full `World` access — then attach them to UI elements as components.

## Features

- **Click / Right-click / Hover / Press** — one component each, zero boilerplate
- **Drag & drop** — state machine with threshold, ghost visual, drop targets
- **Rich tooltips** — builder API with sections, stat diffs, key-value rows
- **Tabs** — `TabGroup` / `Tab` / `TabContent` with automatic visibility
- **Progress bars** — health / mana / stamina presets, spawn helper
- **Visual feedback** — `InteractiveVisual` auto-colors elements on hover/press/select/disable
- **Border feedback** — `BorderStyle` for separate border color states

## Quick start

```rust
use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

struct IncrementAction;

impl UiAction for IncrementAction {
    fn execute(&self, world: &mut World) {
        // Full World access — read/write any resource or entity
        world.resource_mut::<Counter>().0 += 1;
    }
}

#[derive(Resource, Default)]
struct Counter(i32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node { ..default() })
        .with_children(|parent| {
            // One-liner button with action + visual feedback
            parent.spawn_button(IncrementAction, "+1");
        });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<Counter>()
        .add_systems(Startup, setup)
        .run();
}
```

## Drag & drop

```rust
use bevy_ui_actions::prelude::*;

// Mark elements
entity.insert((Draggable, OnDragStart::new(MyStartAction)));
target.insert((DropTarget, OnDrop::new(MyDropAction)));

// In your action, read DragState to know what was dragged where
impl UiAction for MyDropAction {
    fn execute(&self, world: &mut World) {
        let state = world.resource::<DragState>();
        let dragged = state.dragging;    // source entity
        let target = state.drop_target;  // target entity
    }
}
```

## Rich tooltips

```rust
let tooltip = Tooltip::builder()
    .title("Iron Sword")
    .subtitle("Weapon - Main Hand")
    .separator()
    .stat_diff("Damage", "12", StatDiff::Better(4.0))
    .stat_diff("Speed", "1.0x", StatDiff::Neutral)
    .separator()
    .text("A reliable iron sword.")
    .key_value("Weight", "3.5")
    .build();

entity.insert(tooltip);
```

## Tabs

```rust
commands.spawn(TabGroup::new(0)).with_children(|group| {
    // Tab buttons
    group.spawn((Tab::new(0), Button, InteractiveVisual, VisualStyle::tab()));
    group.spawn((Tab::new(1), Button, InteractiveVisual, VisualStyle::tab()));
    // Content panels
    group.spawn(TabContent::new(0)); // visible when tab 0 is active
    group.spawn(TabContent::new(1)); // visible when tab 1 is active
});
```

## Examples

```sh
cargo run --example counter          # Basic click actions
cargo run --example menu             # State management + tooltips
cargo run --example drag_drop        # Drag & drop with ghost
cargo run --example inventory_demo   # Item movement between slots
cargo run --example progress_bar     # HP/MP/SP bars
cargo run --example rich_tooltip     # Stat comparison tooltips
cargo run --example right_click      # Left + right click actions
cargo run --example selection        # Grid selection with BorderStyle
cargo run --example tabs             # Tab switching
```

## Compatibility

| bevy_ui_actions | Bevy  |
|-----------------|-------|
| 0.1             | 0.16  |

## License

MIT
