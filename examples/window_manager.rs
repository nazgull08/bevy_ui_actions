//! Floating window manager demo — multiple windows on screen at once.
//!
//! Multiple draggable, focusable windows. Drag a window by its title bar; click
//! anywhere on a window to bring it to the front; close via the title-bar button
//! or by pressing `Escape` (closes the focused window).
//!
//! Demonstrates:
//! - `spawn_window(WindowConfig, content)` — the floating window widget.
//! - Title-bar drag (`WindowDragState`) + click-to-front (`WindowManager`).
//! - Close button + `Escape`; `WindowConfig::closable(false)` opts out.
//! - Overlapping windows; Bevy resolves overlap via z-index + FocusPolicy.
//!
//! Run: `cargo run --example window_manager -p bevy_ui_actions`

use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        // --- Latency experiment (app-level, not part of the widget) ---
        // Window drag trails the cursor because of Bevy's frame pipeline:
        //   pipelined rendering (renders 1 frame behind) + VSync present (Fifo).
        // AutoNoVsync picks the lowest-latency present mode the GPU supports
        // (Immediate / Mailbox; may tear); disabling pipelined rendering removes
        // the extra render-behind frame. Mailbox isn't available on every backend
        // (e.g. NVIDIA/X11/Vulkan), so AutoNoVsync is the portable choice.
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<PipelinedRenderingPlugin>(),
        )
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Positions chosen to overlap, so click-to-front is visible.
    // These windows use explicit `with_size` to demonstrate fixed sizing (opt-in);
    // the default is content-driven (see the container_transfer example).
    commands.spawn_window(
        WindowConfig::new("Inventory", Vec2::new(100.0, 80.0))
            .with_size(Val::Px(280.0), Val::Px(220.0)),
        |c| {
            c.ui_text(TextRole::Body, "Drag me by the title bar.");
            c.ui_text(TextRole::Label, "Click any window to raise it.");
        },
    );

    // Non-closable: no close button, ignored by Escape.
    commands.spawn_window(
        WindowConfig::new("Stats", Vec2::new(260.0, 170.0))
            .with_size(Val::Px(260.0), Val::Px(200.0))
            .closable(false),
        |c| {
            c.ui_text(TextRole::Body, "STR  10");
            c.ui_text(TextRole::Body, "DEX  12");
            c.ui_text(TextRole::Body, "INT  8");
            c.ui_text(TextRole::Label, "(this window can't be closed)");
        },
    );

    commands.spawn_window(
        WindowConfig::new("Log", Vec2::new(180.0, 280.0)).with_size(Val::Px(320.0), Val::Px(180.0)),
        |c| {
            c.ui_text(TextRole::Body, "Welcome to the window manager demo.");
            c.ui_text(TextRole::Body, "These three windows overlap.");
        },
    );
}
