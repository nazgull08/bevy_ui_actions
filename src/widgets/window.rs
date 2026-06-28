//! Floating window widget — a movable, focusable container.
//!
//! A window is an absolutely-positioned panel with a draggable title bar and a
//! content area. Multiple windows coexist (modeless): Bevy's native picking
//! (`GlobalZIndex` + `FocusPolicy::Block` + `Interaction`) resolves overlap, so
//! the topmost window blocks interaction where it covers others.
//!
//! # Limitations (tracked weaknesses, see `tasks/window_manager/EPIC_WINDOW_MANAGER.md`)
//!
//! - **HiDPI:** hit-testing is scale-factor aware, but only the primary window's
//!   cursor is used — multi-OS-window / multi-UI-camera setups are not supported.
//! - **No keyboard focus routing:** modeless model gives pointer input to all
//!   windows; routing shortcuts/ESC to the focused window is not implemented.
//! - **No resize handles.**

use crate::core::{is_in_scope, TextRole, UiInputScope, UiTextExt, ZLayer};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

/// Marker: the root entity of a floating window.
#[derive(Component)]
pub struct UiWindow;

/// Marker: the title bar of a window. Acts as the drag handle for moving.
///
/// Holds a back-reference to the window root so move/focus systems can act on
/// the whole window from an interaction on the bar.
#[derive(Component)]
pub struct WindowTitleBar {
    pub window: Entity,
}

/// Marker: the content area of a window (where caller children live).
#[derive(Component)]
pub struct WindowContent {
    pub window: Entity,
}

/// Marker: a window's close button.
#[derive(Component)]
pub struct WindowCloseButton {
    pub window: Entity,
}

/// Marker on a window root: its title bar can be dragged to move the window.
///
/// Inserted by [`SpawnWindowExt::spawn_window`] when `WindowConfig::movable` is set.
#[derive(Component)]
pub struct WindowMovable;

/// Marker on a window root: it can be closed (close button + `Escape`).
///
/// Inserted by [`SpawnWindowExt::spawn_window`] when `WindowConfig::closable` is set.
#[derive(Component)]
pub struct WindowClosable;

/// An in-progress window move (title-bar drag).
pub struct WindowDrag {
    pub window: Entity,
    /// Cursor offset from the window's top-left at grab time, in pixels.
    pub grab_offset: Vec2,
}

/// Global state for moving a window by its title bar.
///
/// Separate from [`crate::interactions::DragState`] (item drag-and-drop): items
/// move via `Draggable`, windows move via their title bar.
#[derive(Resource, Default)]
pub struct WindowDragState {
    pub active: Option<WindowDrag>,
}

/// Configuration for a floating window.
///
/// Use [`WindowConfig::new`] then chain builders, or set fields directly.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    /// Top-left position in logical pixels (absolute, from the screen origin).
    pub position: Vec2,
    /// Natural size. Defaults to `Auto` (content-driven); set explicitly via
    /// [`WindowConfig::with_size`] to pin a fixed size.
    pub width: Val,
    pub height: Val,
    /// Lower bound on size, so a content-driven window stays usable (title bar,
    /// close button) even with little content. Mirrors `PanelConfig`.
    pub min_width: Val,
    pub min_height: Val,
    /// Whether the title bar can be dragged to move the window. (Used from Step 2.)
    pub movable: bool,
    /// Whether a close button is shown in the title bar. (Used from Step 4.)
    pub closable: bool,
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub title_bar_color: Color,
    /// Inner padding of the content area, in pixels.
    pub padding: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            position: Vec2::new(100.0, 100.0),
            // Content-driven by default (industry-standard: natural size + min floor).
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Px(180.0),
            min_height: Val::Auto,
            movable: true,
            closable: true,
            background: Color::srgb(0.12, 0.12, 0.15),
            border_color: Color::srgb(0.3, 0.3, 0.35),
            border_width: 2.0,
            title_bar_color: Color::srgb(0.2, 0.2, 0.28),
            padding: 12.0,
        }
    }
}

impl WindowConfig {
    /// New config with a title and top-left position. Other fields default.
    pub fn new(title: impl Into<String>, position: Vec2) -> Self {
        Self {
            title: title.into(),
            position,
            ..Self::default()
        }
    }

    /// Pin an explicit (fixed) window size. Opt-in override of the content-driven default.
    pub fn with_size(mut self, width: Val, height: Val) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the minimum size floor for a content-driven window.
    pub fn with_min_size(mut self, min_width: Val, min_height: Val) -> Self {
        self.min_width = min_width;
        self.min_height = min_height;
        self
    }

    /// Set whether the window can be moved by dragging its title bar.
    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    /// Set whether the window shows a close button.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// Extension trait for spawning floating windows.
pub trait SpawnWindowExt {
    /// Spawn a floating window. `content` fills the window's content area.
    ///
    /// Returns the window **root** entity (useful for focus/close management).
    fn spawn_window(
        &mut self,
        config: WindowConfig,
        content: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> Entity;
}

impl SpawnWindowExt for Commands<'_, '_> {
    fn spawn_window(
        &mut self,
        config: WindowConfig,
        content: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> Entity {
        let root = self
            .spawn((
                UiWindow,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(config.position.x),
                    top: Val::Px(config.position.y),
                    width: config.width,
                    height: config.height,
                    min_width: config.min_width,
                    min_height: config.min_height,
                    flex_direction: FlexDirection::Column,
                    // Stretch children (title bar, content) to the window's width.
                    // Works with content-driven width — unlike a percentage, which
                    // resolves to auto against an indefinite (Auto) parent width.
                    align_items: AlignItems::Stretch,
                    border: UiRect::all(Val::Px(config.border_width)),
                    ..default()
                },
                BackgroundColor(config.background),
                BorderColor(config.border_color),
                // Block clicks from passing through to whatever is behind the window.
                FocusPolicy::Block,
                GlobalZIndex(0),
                Interaction::None,
            ))
            .id();

        if config.movable {
            self.entity(root).insert(WindowMovable);
        }
        if config.closable {
            self.entity(root).insert(WindowClosable);
        }

        let title = config.title.clone();
        let title_bar_color = config.title_bar_color;
        let padding = config.padding;
        let closable = config.closable;

        self.entity(root).with_children(|parent| {
            // Title bar — drag handle (move/focus wired in later steps).
            parent
                .spawn((
                    WindowTitleBar { window: root },
                    Node {
                        // No explicit width: align_items: Stretch on the root fills
                        // the window width (content-driven safe).
                        padding: UiRect::all(Val::Px(6.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    BackgroundColor(title_bar_color),
                    Interaction::None,
                ))
                .with_children(|bar| {
                    // Title stretches to fill the bar, pushing the close button to the edge.
                    bar.ui_text(TextRole::Heading, title).insert(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                    if closable {
                        bar.spawn((
                            WindowCloseButton { window: root },
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                            Interaction::None,
                        ))
                        .with_children(|btn| {
                            btn.ui_text(TextRole::Label, "X");
                        });
                    }
                });

            // Content area — caller children go here.
            parent
                .spawn((
                    WindowContent { window: root },
                    Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(padding)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(content);
        });

        root
    }
}

/// Extract the pixel value from a `Val`, treating non-pixel values as 0.
///
/// Windows are spawned with `Val::Px` left/top, so this is exact for them.
fn val_px(val: Val) -> f32 {
    match val {
        Val::Px(px) => px,
        _ => 0.0,
    }
}

/// Move a window by dragging its title bar.
///
/// On left-press over a movable window's title bar, records the grab offset; while
/// held, follows the cursor; releases on mouse up. Honors [`WindowMovable`].
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn window_move_system(
    mut drag: ResMut<WindowDragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    bars: Query<(&WindowTitleBar, &Interaction)>,
    movable: Query<(), With<WindowMovable>>,
    mut nodes: Query<(&mut Node, &ComputedNode), With<UiWindow>>,
    scope: Option<Res<UiInputScope>>,
    scope_parents: Query<&ChildOf>,
) {
    let cursor = windows.single().ok().and_then(|w| w.cursor_position());

    // Begin a move: press over a movable window's title bar.
    if drag.active.is_none() && mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor) = cursor {
            for (bar, interaction) in &bars {
                if !matches!(interaction, Interaction::Pressed | Interaction::Hovered) {
                    continue;
                }
                if !movable.contains(bar.window) {
                    continue;
                }
                // Respect a modal/overlay input trap (e.g. a modal in front).
                if let Some(ref scope) = scope {
                    if !is_in_scope(bar.window, scope, &scope_parents) {
                        continue;
                    }
                }
                if let Ok((node, _)) = nodes.get(bar.window) {
                    let top_left = Vec2::new(val_px(node.left), val_px(node.top));
                    drag.active = Some(WindowDrag {
                        window: bar.window,
                        grab_offset: cursor - top_left,
                    });
                }
                break;
            }
        }
    }

    // Continue or end the move.
    let Some(active) = &drag.active else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        drag.active = None;
        return;
    }

    if let (Some(cursor), Ok((mut node, computed))) = (cursor, nodes.get_mut(active.window)) {
        let mut pos = cursor - active.grab_offset;
        // Clamp so the window stays fully on screen (title bar never lost).
        if let Ok(win) = windows.single() {
            let screen = Vec2::new(win.width(), win.height());
            let size = computed.size() * computed.inverse_scale_factor();
            let max = (screen - size).max(Vec2::ZERO);
            pos = pos.clamp(Vec2::ZERO, max);
        }
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
}

/// Lowest `GlobalZIndex` assigned to a window. Windows occupy
/// `[WINDOW_Z_BASE, WINDOW_Z_BASE + window_count)`.
///
/// Kept below the library's overlays (see [`ZLayer`]): tooltip, dialogue, modal,
/// drag ghost.
pub const WINDOW_Z_BASE: i32 = ZLayer::Windows.z();

/// Marker on the currently focused (top-most) window. Useful for styling.
#[derive(Component)]
pub struct WindowFocused;

/// Tracks window stacking / focus order. Tail of `focus_stack` = top-most window.
#[derive(Resource, Default)]
pub struct WindowManager {
    pub focus_stack: Vec<Entity>,
}

/// Point-in-rect test for a window, accounting for the UI scale factor.
///
/// `cursor_position()` is logical pixels; UI `GlobalTransform` and
/// `ComputedNode::size()` are physical. We scale the cursor to physical space.
fn cursor_in_window(cursor: Vec2, transform: &GlobalTransform, computed: &ComputedNode) -> bool {
    let scale_factor = 1.0 / computed.inverse_scale_factor();
    let cursor_phys = cursor * scale_factor;
    let center = transform.translation().truncate();
    let half = computed.size() / 2.0;
    cursor_phys.x >= center.x - half.x
        && cursor_phys.x <= center.x + half.x
        && cursor_phys.y >= center.y - half.y
        && cursor_phys.y <= center.y + half.y
}

/// Register newly spawned windows into the manager (appended on top).
pub(crate) fn register_windows(
    mut manager: ResMut<WindowManager>,
    new_windows: Query<Entity, Added<UiWindow>>,
) {
    for entity in &new_windows {
        if !manager.focus_stack.contains(&entity) {
            manager.focus_stack.push(entity);
        }
    }
}

/// Drop despawned windows from the manager's stack.
pub(crate) fn cleanup_windows(
    mut manager: ResMut<WindowManager>,
    mut removed: RemovedComponents<UiWindow>,
) {
    let gone: Vec<Entity> = removed.read().collect();
    if gone.is_empty() {
        return;
    }
    manager.focus_stack.retain(|e| !gone.contains(e));
}

/// Click-to-front: a left-press inside a window raises it to the top of the stack.
///
/// Picks the front-most window under the cursor (modeless; Bevy's own picking
/// already blocks clicks to covered windows, but we hit-test to decide *who* to
/// raise). Honors [`UiInputScope`].
#[allow(clippy::type_complexity)]
pub(crate) fn window_focus_system(
    mut manager: ResMut<WindowManager>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    win_nodes: Query<(&GlobalTransform, &ComputedNode), With<UiWindow>>,
    scope: Option<Res<UiInputScope>>,
    scope_parents: Query<&ChildOf>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = windows.single().ok().and_then(|w| w.cursor_position()) else {
        return;
    };

    // Front-to-back: tail of the stack is on top.
    let mut hit: Option<Entity> = None;
    for &win in manager.focus_stack.iter().rev() {
        if let Some(ref scope) = scope {
            if !is_in_scope(win, scope, &scope_parents) {
                continue;
            }
        }
        if let Ok((transform, computed)) = win_nodes.get(win) {
            if cursor_in_window(cursor, transform, computed) {
                hit = Some(win);
                break;
            }
        }
    }

    if let Some(win) = hit {
        if manager.focus_stack.last() != Some(&win) {
            manager.focus_stack.retain(|&e| e != win);
            manager.focus_stack.push(win);
        }
    }
}

/// Write `GlobalZIndex` from stack order and maintain the [`WindowFocused`] marker.
///
/// Runs only when the manager changed (window added/removed/raised).
pub(crate) fn apply_window_z(
    manager: Res<WindowManager>,
    mut z_query: Query<&mut GlobalZIndex, With<UiWindow>>,
    focused: Query<Entity, With<WindowFocused>>,
    mut commands: Commands,
) {
    if !manager.is_changed() {
        return;
    }

    for (i, &win) in manager.focus_stack.iter().enumerate() {
        if let Ok(mut z) = z_query.get_mut(win) {
            let new_z = WINDOW_Z_BASE + i as i32;
            if z.0 != new_z {
                z.0 = new_z;
            }
        }
    }

    let top = manager.focus_stack.last().copied();
    for entity in &focused {
        if Some(entity) != top {
            commands.entity(entity).remove::<WindowFocused>();
        }
    }
    if let Some(top) = top {
        commands.entity(top).insert(WindowFocused);
    }
}

/// Despawn a window when its close button is clicked. Honors [`UiInputScope`].
///
/// Despawn is recursive (Bevy 0.16), so the whole window subtree is removed;
/// `cleanup_windows` then drops it from the manager next frame.
#[allow(clippy::type_complexity)]
pub(crate) fn window_close_system(
    mut commands: Commands,
    buttons: Query<(&WindowCloseButton, &Interaction), Changed<Interaction>>,
    scope: Option<Res<UiInputScope>>,
    scope_parents: Query<&ChildOf>,
) {
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(ref scope) = scope {
            if !is_in_scope(button.window, scope, &scope_parents) {
                continue;
            }
        }
        commands.entity(button.window).despawn();
    }
}

/// Close the focused window on `Escape`.
///
/// Opinionated UX: skipped while a [`UiInputScope`] is active (then `Escape`
/// belongs to the modal/dialogue that owns the scope, not to windows).
pub(crate) fn window_close_on_escape(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    focused: Query<Entity, (With<WindowFocused>, With<WindowClosable>)>,
    scope: Option<Res<UiInputScope>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if scope.is_some() {
        return;
    }
    if let Some(window) = focused.iter().next() {
        commands.entity(window).despawn();
    }
}
