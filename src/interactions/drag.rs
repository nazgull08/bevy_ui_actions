use crate::core::UiAction;
use crate::widgets::Disabled;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use std::sync::Arc;

/// Movement threshold in pixels before a drag begins.
const DRAG_THRESHOLD: f32 = 5.0;

/// Drag-and-drop state machine phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragPhase {
    #[default]
    Idle,
    Pending,
    Active,
}

/// Marker: this element can be dragged.
#[derive(Component)]
pub struct Draggable;

/// Marker: this element accepts drops.
#[derive(Component)]
pub struct DropTarget;

/// Action fired when a drag begins.
#[derive(Component)]
pub struct OnDragStart {
    pub(crate) action: Arc<dyn UiAction>,
}

impl OnDragStart {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }
}

/// Action fired on a successful drop onto a target.
#[derive(Component)]
pub struct OnDrop {
    pub(crate) action: Arc<dyn UiAction>,
}

impl OnDrop {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }
}

/// Action fired when a drag is cancelled (released outside any target).
#[derive(Component)]
pub struct OnDragCancel {
    pub(crate) action: Arc<dyn UiAction>,
}

impl OnDragCancel {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }
}

/// Global drag-and-drop state.
#[derive(Resource, Default)]
pub struct DragState {
    pub phase: DragPhase,
    pub dragging: Option<Entity>,
    pub drop_target: Option<Entity>,
    pub start_pos: Vec2,
    pub ghost_entity: Option<Entity>,
}

impl DragState {
    pub fn clear(&mut self) {
        self.phase = DragPhase::Idle;
        self.dragging = None;
        self.drop_target = None;
        self.start_pos = Vec2::ZERO;
        self.ghost_entity = None;
    }

    pub fn is_dragging(&self) -> bool {
        self.phase == DragPhase::Active
    }
}

/// Marker for the drag ghost UI entity.
#[derive(Component)]
pub struct DragGhost;

/// Visual style for the drag ghost.
#[derive(Resource)]
pub struct DragGhostStyle {
    pub background: Color,
    pub size: f32,
    pub opacity: f32,
}

impl Default for DragGhostStyle {
    fn default() -> Self {
        Self {
            background: Color::srgba(0.5, 0.5, 0.8, 0.7),
            size: 50.0,
            opacity: 0.7,
        }
    }
}

/// Visual source for the drag ghost.
enum GhostVisual {
    /// Use an image from the dragged element.
    Image(Handle<Image>),
    /// Use a solid background color.
    Color(Color),
}

// ============ State Machine System ============

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn drag_system(
    mut drag_state: ResMut<DragState>,
    ghost_style: Res<DragGhostStyle>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    draggables: Query<
        (
            Entity,
            &Interaction,
            Option<&OnDragStart>,
            Option<&OnDragCancel>,
            Option<&BackgroundColor>,
            Option<&Children>,
        ),
        (With<Draggable>, Without<Disabled>),
    >,
    image_query: Query<&ImageNode>,
    drop_targets: Query<(Entity, &Interaction, Option<&OnDrop>), With<DropTarget>>,
    mut ghost_query: Query<&mut Node, With<DragGhost>>,
    mut commands: Commands,
) {
    let cursor_pos = windows.single().ok().and_then(|w| w.cursor_position());

    match drag_state.phase {
        // ========== IDLE ==========
        DragPhase::Idle => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }

            let Some(cursor) = cursor_pos else { return };

            for (entity, interaction, _, _, _, _) in &draggables {
                if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
                    drag_state.phase = DragPhase::Pending;
                    drag_state.dragging = Some(entity);
                    drag_state.start_pos = cursor;
                    return;
                }
            }
        }

        // ========== PENDING ==========
        DragPhase::Pending => {
            if mouse.just_released(MouseButton::Left) {
                drag_state.clear();
                return;
            }

            let Some(cursor) = cursor_pos else { return };
            let Some(dragged_entity) = drag_state.dragging else {
                drag_state.clear();
                return;
            };

            let distance = cursor.distance(drag_state.start_pos);
            if distance < DRAG_THRESHOLD {
                return;
            }

            drag_state.phase = DragPhase::Active;

            // Determine ghost visual
            let ghost_visual =
                find_ghost_visual(dragged_entity, &draggables, &image_query, &ghost_style);

            // Spawn ghost
            let ghost = spawn_ghost(&mut commands, cursor, &ghost_style, ghost_visual);
            drag_state.ghost_entity = Some(ghost);

            // Fire OnDragStart if present
            if let Ok((_, _, Some(on_start), _, _, _)) = draggables.get(dragged_entity) {
                let action = on_start.action.clone();
                commands.queue(move |world: &mut World| {
                    action.execute(world);
                });
            }
        }

        // ========== ACTIVE ==========
        DragPhase::Active => {
            let Some(dragged_entity) = drag_state.dragging else {
                drag_state.clear();
                return;
            };

            // Update ghost position
            if let Some(cursor) = cursor_pos {
                if let Some(ghost_entity) = drag_state.ghost_entity {
                    if let Ok(mut node) = ghost_query.get_mut(ghost_entity) {
                        node.left = Val::Px(cursor.x - ghost_style.size / 2.0);
                        node.top = Val::Px(cursor.y - ghost_style.size / 2.0);
                    }
                }
            }

            if !mouse.just_released(MouseButton::Left) {
                return;
            }

            // Despawn ghost
            if let Some(ghost) = drag_state.ghost_entity.take() {
                commands.entity(ghost).despawn();
            }

            // Find drop target
            let mut found_target: Option<(Entity, Option<Arc<dyn UiAction>>)> = None;

            for (target_entity, interaction, on_drop) in &drop_targets {
                if target_entity == dragged_entity {
                    continue;
                }

                if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
                    found_target = Some((target_entity, on_drop.map(|d| d.action.clone())));
                    break;
                }
            }

            if let Some((target_entity, action)) = found_target {
                drag_state.drop_target = Some(target_entity);

                if let Some(action) = action {
                    let dragging = drag_state.dragging;
                    let drop_target = drag_state.drop_target;

                    commands.queue(move |world: &mut World| {
                        {
                            let mut state = world.resource_mut::<DragState>();
                            state.dragging = dragging;
                            state.drop_target = drop_target;
                        }
                        action.execute(world);
                    });
                }
            } else {
                // Cancel drag
                if let Ok((_, _, _, Some(on_cancel), _, _)) = draggables.get(dragged_entity) {
                    let action = on_cancel.action.clone();
                    let dragging = drag_state.dragging; // Save before clear

                    commands.queue(move |world: &mut World| {
                        // Restore dragging for the action
                        {
                            let mut state = world.resource_mut::<DragState>();
                            state.dragging = dragging;
                        }
                        action.execute(world);
                    });
                }
            }

            drag_state.clear();
        }
    }
}

/// Find a visual for the ghost (image or color).
#[allow(clippy::type_complexity)]
fn find_ghost_visual(
    entity: Entity,
    draggables: &Query<
        (
            Entity,
            &Interaction,
            Option<&OnDragStart>,
            Option<&OnDragCancel>,
            Option<&BackgroundColor>,
            Option<&Children>,
        ),
        (With<Draggable>, Without<Disabled>),
    >,
    image_query: &Query<&ImageNode>,
    ghost_style: &DragGhostStyle,
) -> GhostVisual {
    let Ok((_, _, _, _, bg_color, children)) = draggables.get(entity) else {
        return GhostVisual::Color(ghost_style.background);
    };

    // 1. Check ImageNode on the element itself
    if let Ok(image_node) = image_query.get(entity) {
        return GhostVisual::Image(image_node.image.clone());
    }

    // 2. Look for ImageNode in children (for slots with icons)
    if let Some(children) = children {
        for child in children.iter() {
            if let Ok(image_node) = image_query.get(child) {
                // Found an image child
                return GhostVisual::Image(image_node.image.clone());
            }
        }
    }

    // 3. Fall back to BackgroundColor
    if let Some(bg) = bg_color {
        let c = bg.0.to_srgba();
        return GhostVisual::Color(Color::srgba(c.red, c.green, c.blue, ghost_style.opacity));
    }

    // 4. Fallback
    GhostVisual::Color(ghost_style.background)
}

/// Spawn a ghost entity at the cursor position.
fn spawn_ghost(
    commands: &mut Commands,
    cursor: Vec2,
    style: &DragGhostStyle,
    visual: GhostVisual,
) -> Entity {
    let base_node = Node {
        position_type: PositionType::Absolute,
        left: Val::Px(cursor.x - style.size / 2.0),
        top: Val::Px(cursor.y - style.size / 2.0),
        width: Val::Px(style.size),
        height: Val::Px(style.size),
        ..default()
    };

    match visual {
        GhostVisual::Image(handle) => commands
            .spawn((
                DragGhost,
                base_node,
                ImageNode {
                    image: handle,
                    ..default()
                },
                GlobalZIndex(999),
                FocusPolicy::Pass,
            ))
            .id(),
        GhostVisual::Color(color) => commands
            .spawn((
                DragGhost,
                base_node,
                BackgroundColor(color),
                GlobalZIndex(999),
                FocusPolicy::Pass,
            ))
            .id(),
    }
}

/// Run condition: returns true when any [`Draggable`] entities exist.
pub fn has_draggables(query: Query<(), With<Draggable>>) -> bool {
    !query.is_empty()
}
