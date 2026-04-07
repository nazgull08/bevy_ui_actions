use bevy::prelude::*;
use std::sync::Arc;

use crate::core::{TextRole, UiAction, UiInputScope, UiTextExt};
use crate::interactions::OnClick;
use crate::widgets::visual::InteractiveVisual;

// ============================================================
// Types
// ============================================================

/// Visual configuration for modals.
#[derive(Resource, Clone)]
pub struct ModalStyle {
    pub backdrop_color: Color,
    pub panel_background: Color,
    pub panel_border: Color,
    pub panel_border_width: f32,
    pub panel_padding: f32,
    pub panel_gap: f32,
    pub panel_min_width: f32,
    pub panel_max_width: f32,
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    pub confirm_color: Color,
    pub cancel_color: Color,
}

impl Default for ModalStyle {
    fn default() -> Self {
        Self {
            backdrop_color: Color::srgba(0.0, 0.0, 0.0, 0.6),
            panel_background: Color::srgb(0.12, 0.12, 0.15),
            panel_border: Color::srgb(0.4, 0.4, 0.45),
            panel_border_width: 2.0,
            panel_padding: 25.0,
            panel_gap: 15.0,
            panel_min_width: 300.0,
            panel_max_width: 500.0,
            button_padding_x: 24.0,
            button_padding_y: 10.0,
            confirm_color: Color::srgb(0.2, 0.35, 0.2),
            cancel_color: Color::srgb(0.35, 0.2, 0.2),
        }
    }
}

/// Request to show a modal. Queued in `ModalQueue` resource.
///
/// Content is always a builder closure that receives the panel and the modal style.
/// Use `ModalRequest::confirm()` for a pre-built confirm dialog,
/// or `ModalRequest::new()` for fully custom content.
type ModalBuilder = Box<dyn FnOnce(&mut ChildSpawnerCommands, &ModalStyle) + Send + Sync>;

pub struct ModalRequest {
    content: ModalBuilder,
    pub on_confirm: Option<Arc<dyn UiAction>>,
    pub on_cancel: Option<Arc<dyn UiAction>>,
    pub dismissable: bool,
}

impl ModalRequest {
    /// Create a modal with fully custom content.
    ///
    /// The builder receives the panel's `ChildSpawnerCommands` and the `ModalStyle`.
    /// Use `spawn_modal_button` to add styled buttons, and `DismissModal` action to close.
    pub fn new(
        builder: impl FnOnce(&mut ChildSpawnerCommands, &ModalStyle) + Send + Sync + 'static,
    ) -> Self {
        Self {
            content: Box::new(builder),
            on_confirm: None,
            on_cancel: None,
            dismissable: true,
        }
    }

    /// Create a confirm dialog with title, message, and Confirm/Cancel buttons.
    ///
    /// This is sugar over `new()`. Buttons use `ModalStyle` colors.
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        let title = title.into();
        let message = message.into();
        Self::new(move |panel, style| {
            panel.ui_text(TextRole::Heading, &title);
            panel.ui_text(TextRole::Body, &message);

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|row| {
                    spawn_modal_button(
                        row,
                        "Confirm",
                        style.confirm_color,
                        style.button_padding_x,
                        style.button_padding_y,
                        DismissModal(true),
                    );
                    spawn_modal_button(
                        row,
                        "Cancel",
                        style.cancel_color,
                        style.button_padding_x,
                        style.button_padding_y,
                        DismissModal(false),
                    );
                });
        })
    }

    /// Set confirm action (chainable).
    pub fn with_confirm(mut self, action: impl UiAction) -> Self {
        self.on_confirm = Some(Arc::new(action));
        self
    }

    /// Set cancel action (chainable).
    pub fn with_cancel(mut self, action: impl UiAction) -> Self {
        self.on_cancel = Some(Arc::new(action));
        self
    }

    /// Set dismissable flag (chainable).
    pub fn with_dismissable(mut self, dismissable: bool) -> Self {
        self.dismissable = dismissable;
        self
    }
}

/// Resource queue for modal requests. Allows `FnOnce` content (can't go through events).
#[derive(Resource, Default)]
pub struct ModalQueue {
    pub(crate) pending: Vec<ModalRequest>,
}

impl ModalQueue {
    /// Queue a modal to be shown next frame.
    pub fn show(&mut self, request: ModalRequest) {
        self.pending.push(request);
    }
}

/// Event: dismiss the current modal.
#[derive(Event)]
pub struct DismissModalEvent {
    /// If true, fires on_confirm action. If false, fires on_cancel.
    pub confirmed: bool,
}

// ============================================================
// Components
// ============================================================

/// Marker on the modal root (backdrop).
#[derive(Component)]
pub struct Modal {
    pub on_confirm: Option<Arc<dyn UiAction>>,
    pub on_cancel: Option<Arc<dyn UiAction>>,
    pub dismissable: bool,
}

/// Marker on the backdrop node (for click detection).
#[derive(Component)]
pub struct ModalBackdrop;

/// Marker on the modal panel (content area).
#[derive(Component)]
pub struct ModalPanel;

/// Marker on the modal root: hidden until layout is computed.
/// Removed in `PostUpdate` after Bevy's layout pass, making the modal visible.
#[derive(Component)]
pub(crate) struct ModalLayoutPending;


// ============================================================
// Public Helpers
// ============================================================

/// Action that dismisses the current modal.
///
/// `DismissModal(true)` fires `on_confirm`, `DismissModal(false)` fires `on_cancel`.
pub struct DismissModal(pub bool);

impl UiAction for DismissModal {
    fn execute(&self, world: &mut World) {
        world.send_event(DismissModalEvent { confirmed: self.0 });
    }
}

/// Spawn a styled button inside a modal panel.
///
/// Use with `DismissModal` action to close the modal:
/// ```ignore
/// spawn_modal_button(row, "OK", style.confirm_color, style.button_padding_x, style.button_padding_y, DismissModal(true));
/// ```
pub fn spawn_modal_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    color: Color,
    padding_x: f32,
    padding_y: f32,
    action: impl UiAction,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(padding_x), Val::Px(padding_y)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            OnClick::new(action),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text(TextRole::Button, label);
        });
}

// ============================================================
// Systems
// ============================================================

/// Processes queued modal requests.
pub(crate) fn process_modal_queue(
    mut queue: ResMut<ModalQueue>,
    mut commands: Commands,
    style: Res<ModalStyle>,
    existing: Query<Entity, With<Modal>>,
) {
    let Some(request) = queue.pending.pop() else {
        return;
    };
    // Clear remaining (one modal at a time)
    queue.pending.clear();

    // Despawn existing modal
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let modal_entity = spawn_modal(&mut commands, &style, request);
    commands.insert_resource(UiInputScope { root: modal_entity });
}

fn spawn_modal(commands: &mut Commands, style: &ModalStyle, request: ModalRequest) -> Entity {
    let on_confirm = request.on_confirm;
    let on_cancel = request.on_cancel;
    let dismissable = request.dismissable;
    let content = request.content;

    let modal_entity = commands
        .spawn((
            Modal {
                on_confirm,
                on_cancel,
                dismissable,
            },
            ModalBackdrop,
            ModalLayoutPending,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(style.backdrop_color),
            GlobalZIndex(900),
            Interaction::None,
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    ModalPanel,
                    Node {
                        min_width: Val::Px(style.panel_min_width),
                        max_width: Val::Px(style.panel_max_width),
                        padding: UiRect::all(Val::Px(style.panel_padding)),
                        border: UiRect::all(Val::Px(style.panel_border_width)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(style.panel_gap),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(style.panel_background),
                    BorderColor(style.panel_border),
                    Interaction::None,
                ))
                .with_children(|panel| {
                    content(panel, style);
                });
        })
        .id();

    modal_entity
}


/// Reveals modals once panel size has stabilized (prevents size pop).
///
/// Waits until the `ModalPanel`'s `ComputedNode` size is non-zero and
/// unchanged from the previous frame, then removes `ModalLayoutPending`
/// and sets `Visibility::Visible`. This accounts for Bevy's text
/// measurement needing 1-2 frames to settle.
pub(crate) fn reveal_modal_panel(
    mut commands: Commands,
    query: Query<Entity, With<ModalLayoutPending>>,
    panel_query: Query<&ComputedNode, With<ModalPanel>>,
    mut last_size: Local<Vec2>,
) {
    for entity in &query {
        let current_size = panel_query
            .iter()
            .next()
            .map(|c| c.size())
            .unwrap_or(Vec2::ZERO);

        if current_size.x > 0.0 && current_size == *last_size {
            // Size stabilized — safe to reveal
            commands
                .entity(entity)
                .remove::<ModalLayoutPending>()
                .insert(Visibility::Visible);
            *last_size = Vec2::ZERO;
        } else {
            *last_size = current_size;
        }
    }
}


/// Handles ESC key and backdrop clicks to dismiss modal.
pub(crate) fn handle_modal_dismiss(
    keys: Res<ButtonInput<KeyCode>>,
    backdrop_query: Query<(&Interaction, &Modal), With<ModalBackdrop>>,
    panel_query: Query<&Interaction, With<ModalPanel>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut dismiss_events: EventWriter<DismissModalEvent>,
) {
    // ESC to dismiss
    if keys.just_pressed(KeyCode::Escape) {
        for (_, modal) in &backdrop_query {
            if modal.dismissable {
                dismiss_events.write(DismissModalEvent { confirmed: false });
                return;
            }
        }
    }

    // Backdrop click to dismiss (only if panel is NOT hovered)
    if mouse.just_pressed(MouseButton::Left) {
        for (backdrop_interaction, modal) in &backdrop_query {
            if !modal.dismissable {
                continue;
            }

            let panel_interacted = panel_query
                .iter()
                .any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);

            if !panel_interacted
                && (*backdrop_interaction == Interaction::Hovered
                    || *backdrop_interaction == Interaction::Pressed)
            {
                dismiss_events.write(DismissModalEvent { confirmed: false });
                return;
            }
        }
    }
}

/// Processes DismissModalEvent: fires action and despawns.
pub(crate) fn handle_dismiss_event(
    mut events: EventReader<DismissModalEvent>,
    query: Query<(Entity, &Modal)>,
    mut commands: Commands,
    scope: Option<Res<UiInputScope>>,
) {
    for event in events.read() {
        for (entity, modal) in &query {
            let action = if event.confirmed {
                modal.on_confirm.clone()
            } else {
                modal.on_cancel.clone()
            };

            if let Some(action) = action {
                commands.queue(move |world: &mut World| {
                    action.execute(world);
                });
            }

            commands.entity(entity).despawn();

            // Remove input scope when modal is dismissed
            if scope.is_some() {
                commands.remove_resource::<UiInputScope>();
            }
        }
    }
}
