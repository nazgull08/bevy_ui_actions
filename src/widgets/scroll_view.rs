use crate::core::{is_in_scope, UiInputScope};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

// ============================================================
// Types
// ============================================================

/// Scroll axis direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

/// Configuration for spawning a scroll view.
#[derive(Clone, Debug)]
pub struct ScrollViewConfig {
    pub direction: ScrollDirection,
    /// Pixels per mouse wheel "line" event.
    pub scroll_speed: f32,
    pub width: Val,
    pub height: Val,
    pub background: Option<Color>,
    /// Show a scrollbar track + thumb.
    pub show_scrollbar: bool,
    /// Scrollbar track width in pixels.
    pub scrollbar_width: f32,
    /// Scrollbar track background color.
    pub scrollbar_track: Color,
    /// Scrollbar thumb color.
    pub scrollbar_thumb: Color,
}

impl Default for ScrollViewConfig {
    fn default() -> Self {
        Self {
            direction: ScrollDirection::Vertical,
            scroll_speed: 40.0,
            width: Val::Percent(100.0),
            height: Val::Px(300.0),
            background: None,
            show_scrollbar: false,
            scrollbar_width: 12.0,
            scrollbar_track: Color::srgba(0.15, 0.15, 0.18, 0.8),
            scrollbar_thumb: Color::srgba(0.5, 0.5, 0.55, 0.8),
        }
    }
}

/// Marker component on a scrollable container.
#[derive(Component)]
pub struct ScrollView {
    pub direction: ScrollDirection,
    pub scroll_speed: f32,
}

/// Marker on the scrollbar thumb. Points to the ScrollView entity.
#[derive(Component)]
pub struct ScrollbarThumb {
    pub scroll_view: Entity,
}

/// Marker on the scrollbar track. Points to the ScrollView entity.
#[derive(Component)]
pub struct ScrollbarTrack {
    pub scroll_view: Entity,
}

/// Global drag state for scrollbar thumb dragging.
#[derive(Resource, Default)]
pub struct ScrollbarDragState {
    /// Which ScrollView entity is being scrolled via thumb drag.
    pub dragging: Option<Entity>,
    /// Mouse Y at drag start.
    pub start_mouse_y: f32,
    /// ScrollPosition.offset_y at drag start.
    pub start_scroll_offset: f32,
    /// Max scroll value at drag start (for proportional mapping).
    pub max_scroll: f32,
    /// Usable track range (track_height - thumb_height) at drag start.
    pub usable_track: f32,
}

// ============================================================
// Spawn Extension
// ============================================================

/// Extension trait for spawning scroll views.
pub trait SpawnScrollViewExt {
    /// Spawn a scrollable container without scrollbar.
    /// Add children with `.with_children()`.
    fn spawn_scroll_view(&mut self, config: ScrollViewConfig) -> EntityCommands<'_>;

    /// Spawn a scrollable container with scrollbar (wrapper pattern).
    /// Children go into the scroll area via the callback.
    fn spawn_scroll_view_with(
        &mut self,
        config: ScrollViewConfig,
        children: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> Entity;
}

impl SpawnScrollViewExt for ChildSpawnerCommands<'_> {
    fn spawn_scroll_view(&mut self, config: ScrollViewConfig) -> EntityCommands<'_> {
        let (overflow, direction) = build_overflow(config.direction);
        let mut ec = self.spawn((
            Node {
                width: config.width,
                height: config.height,
                overflow,
                flex_direction: direction,
                ..default()
            },
            ScrollPosition::default(),
            ScrollView {
                direction: config.direction,
                scroll_speed: config.scroll_speed,
            },
            Interaction::None,
        ));

        if let Some(color) = config.background {
            ec.insert(BackgroundColor(color));
        }

        ec
    }

    fn spawn_scroll_view_with(
        &mut self,
        config: ScrollViewConfig,
        children: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> Entity {
        if !config.show_scrollbar {
            // No scrollbar — simple spawn, apply children directly
            let mut ec = self.spawn_scroll_view(config);
            ec.with_children(children);
            return ec.id();
        }

        let scrollbar_width = config.scrollbar_width;
        let track_color = config.scrollbar_track;
        let thumb_color = config.scrollbar_thumb;

        // Wrapper row — min_height:0 lets flexbox respect overflow:scroll on children
        let wrapper = self
            .spawn(Node {
                width: config.width,
                height: config.height,
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .id();

        // Scroll container (takes remaining space)
        let (overflow, direction) = build_overflow(config.direction);
        let scroll_node = Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            min_height: Val::Px(0.0),
            overflow,
            flex_direction: direction,
            ..default()
        };

        let scroll_entity = self
            .commands()
            .spawn((
                scroll_node,
                ScrollPosition::default(),
                ScrollView {
                    direction: config.direction,
                    scroll_speed: config.scroll_speed,
                },
                Interaction::None,
            ))
            .id();

        if let Some(color) = config.background {
            self.commands()
                .entity(scroll_entity)
                .insert(BackgroundColor(color));
        }

        // Add user's children to scroll container
        self.commands()
            .entity(scroll_entity)
            .with_children(children);

        // Scrollbar track
        let track_entity = self
            .commands()
            .spawn((
                ScrollbarTrack {
                    scroll_view: scroll_entity,
                },
                Node {
                    width: Val::Px(scrollbar_width),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(track_color),
                Interaction::None,
            ))
            .id();

        // Thumb inside track
        let thumb_entity = self
            .commands()
            .spawn((
                ScrollbarThumb {
                    scroll_view: scroll_entity,
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0), // initial; updated by system
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(thumb_color),
                Interaction::None,
                BorderRadius::all(Val::Px(scrollbar_width / 2.0)),
            ))
            .id();

        // Assemble hierarchy: wrapper → [scroll_container, track → [thumb]]
        self.commands().entity(track_entity).add_child(thumb_entity);
        self.commands()
            .entity(wrapper)
            .add_child(scroll_entity)
            .add_child(track_entity);

        wrapper
    }
}

impl SpawnScrollViewExt for Commands<'_, '_> {
    fn spawn_scroll_view(&mut self, config: ScrollViewConfig) -> EntityCommands<'_> {
        let (overflow, direction) = build_overflow(config.direction);

        let mut ec = self.spawn((
            Node {
                width: config.width,
                height: config.height,
                overflow,
                flex_direction: direction,
                ..default()
            },
            ScrollPosition::default(),
            ScrollView {
                direction: config.direction,
                scroll_speed: config.scroll_speed,
            },
            Interaction::None,
        ));

        if let Some(color) = config.background {
            ec.insert(BackgroundColor(color));
        }

        ec
    }

    fn spawn_scroll_view_with(
        &mut self,
        config: ScrollViewConfig,
        children: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> Entity {
        if !config.show_scrollbar {
            let mut ec = self.spawn_scroll_view(config);
            ec.with_children(children);
            return ec.id();
        }

        let scrollbar_width = config.scrollbar_width;
        let track_color = config.scrollbar_track;
        let thumb_color = config.scrollbar_thumb;

        // Wrapper row — min_height:0 lets flexbox respect overflow:scroll on children
        let wrapper = self
            .spawn(Node {
                width: config.width,
                height: config.height,
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .id();

        // Scroll container
        let (overflow, direction) = build_overflow(config.direction);
        let scroll_entity = self
            .spawn((
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    overflow,
                    flex_direction: direction,
                    ..default()
                },
                ScrollPosition::default(),
                ScrollView {
                    direction: config.direction,
                    scroll_speed: config.scroll_speed,
                },
                Interaction::None,
            ))
            .id();

        if let Some(color) = config.background {
            self.entity(scroll_entity)
                .insert(BackgroundColor(color));
        }

        self.entity(scroll_entity).with_children(children);

        // Scrollbar track
        let track_entity = self
            .spawn((
                ScrollbarTrack {
                    scroll_view: scroll_entity,
                },
                Node {
                    width: Val::Px(scrollbar_width),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(track_color),
                Interaction::None,
            ))
            .id();

        // Thumb
        let thumb_entity = self
            .spawn((
                ScrollbarThumb {
                    scroll_view: scroll_entity,
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(thumb_color),
                Interaction::None,
                BorderRadius::all(Val::Px(scrollbar_width / 2.0)),
            ))
            .id();

        self.entity(track_entity).add_child(thumb_entity);
        self.entity(wrapper)
            .add_child(scroll_entity)
            .add_child(track_entity);

        wrapper
    }
}

fn build_overflow(dir: ScrollDirection) -> (Overflow, FlexDirection) {
    match dir {
        ScrollDirection::Vertical => (
            Overflow {
                x: OverflowAxis::Clip,
                y: OverflowAxis::Scroll,
            },
            FlexDirection::Column,
        ),
        ScrollDirection::Horizontal => (
            Overflow {
                x: OverflowAxis::Scroll,
                y: OverflowAxis::Clip,
            },
            FlexDirection::Row,
        ),
        ScrollDirection::Both => (
            Overflow {
                x: OverflowAxis::Scroll,
                y: OverflowAxis::Scroll,
            },
            FlexDirection::Column,
        ),
    }
}

// ============================================================
// Systems
// ============================================================

/// Reads mouse wheel events and applies scroll to ScrollView under cursor.
///
/// Uses cursor position + node bounds instead of `Interaction` to avoid
/// child elements (buttons, etc.) stealing hover from the scroll container.
pub(crate) fn handle_scroll_input(
    mut wheel_events: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut query: Query<(
        Entity,
        &ScrollView,
        &mut ScrollPosition,
        &GlobalTransform,
        &ComputedNode,
    )>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
) {
    let mut total_x: f32 = 0.0;
    let mut total_y: f32 = 0.0;

    for event in wheel_events.read() {
        let (dx, dy) = match event.unit {
            MouseScrollUnit::Line => (event.x, event.y),
            MouseScrollUnit::Pixel => (event.x / 40.0, event.y / 40.0),
        };
        total_x += dx;
        total_y += dy;
    }

    if total_x == 0.0 && total_y == 0.0 {
        return;
    }

    let Some(cursor) = windows.single().ok().and_then(|w| w.cursor_position()) else {
        return;
    };

    for (entity, scroll_view, mut scroll_pos, transform, computed) in &mut query {
        if !cursor_in_node(cursor, transform, computed) {
            continue;
        }

        if let Some(ref scope) = scope {
            if !is_in_scope(entity, scope, &parents) {
                continue;
            }
        }

        let speed = scroll_view.scroll_speed;

        match scroll_view.direction {
            ScrollDirection::Vertical => {
                scroll_pos.offset_y -= total_y * speed;
            }
            ScrollDirection::Horizontal => {
                scroll_pos.offset_x -= total_x * speed;
            }
            ScrollDirection::Both => {
                scroll_pos.offset_x -= total_x * speed;
                scroll_pos.offset_y -= total_y * speed;
            }
        }

        // Only scroll one container (topmost under cursor)
        break;
    }
}

/// Check if cursor position falls within a UI node's bounds.
fn cursor_in_node(cursor: Vec2, transform: &GlobalTransform, computed: &ComputedNode) -> bool {
    let node_pos = transform.translation().truncate();
    let size = computed.size();
    let half = size / 2.0;
    cursor.x >= node_pos.x - half.x
        && cursor.x <= node_pos.x + half.x
        && cursor.y >= node_pos.y - half.y
        && cursor.y <= node_pos.y + half.y
}

/// Clamps scroll position to valid bounds (0..max_scroll).
pub(crate) fn clamp_scroll_bounds(
    mut query: Query<(&ScrollView, &mut ScrollPosition, &ComputedNode, &Children)>,
    child_nodes: Query<&ComputedNode, Without<ScrollView>>,
) {
    for (scroll_view, mut scroll_pos, viewport_node, children) in &mut query {
        let viewport_size = viewport_node.size();

        // Skip clamping when node is hidden (Display::None → size is 0)
        if viewport_size.x <= 0.0 && viewport_size.y <= 0.0 {
            continue;
        }

        // Calculate total content size from children
        let mut content_width: f32 = 0.0;
        let mut content_height: f32 = 0.0;

        for child in children.iter() {
            if let Ok(child_node) = child_nodes.get(child) {
                let child_size = child_node.size();
                match scroll_view.direction {
                    ScrollDirection::Vertical => {
                        content_height += child_size.y;
                        content_width = content_width.max(child_size.x);
                    }
                    ScrollDirection::Horizontal => {
                        content_width += child_size.x;
                        content_height = content_height.max(child_size.y);
                    }
                    ScrollDirection::Both => {
                        content_height += child_size.y;
                        content_width = content_width.max(child_size.x);
                    }
                }
            }
        }

        let max_scroll_x = (content_width - viewport_size.x).max(0.0);
        let max_scroll_y = (content_height - viewport_size.y).max(0.0);

        scroll_pos.offset_x = scroll_pos.offset_x.clamp(0.0, max_scroll_x);
        scroll_pos.offset_y = scroll_pos.offset_y.clamp(0.0, max_scroll_y);
    }
}

/// Updates scrollbar thumb position and size based on scroll state.
pub(crate) fn update_scrollbar_thumb(
    scroll_query: Query<(&ScrollPosition, &ComputedNode, &Children), With<ScrollView>>,
    child_nodes: Query<&ComputedNode, Without<ScrollView>>,
    track_query: Query<&ComputedNode, With<ScrollbarTrack>>,
    mut thumb_query: Query<(&ScrollbarThumb, &mut Node, &ChildOf)>,
) {
    for (thumb, mut thumb_node, child_of) in &mut thumb_query {
        let Ok((scroll_pos, viewport_node, children)) = scroll_query.get(thumb.scroll_view) else {
            continue;
        };

        // Get track height from thumb's parent (the track)
        let Ok(track_node) = track_query.get(child_of.parent()) else {
            continue;
        };

        let viewport_height = viewport_node.size().y;
        let track_height = track_node.size().y;

        if track_height <= 0.0 || viewport_height <= 0.0 {
            continue;
        }

        // Calculate content height
        let mut content_height: f32 = 0.0;
        for child in children.iter() {
            if let Ok(child_node) = child_nodes.get(child) {
                content_height += child_node.size().y;
            }
        }

        if content_height <= viewport_height {
            // No scrolling needed — hide thumb
            thumb_node.height = Val::Px(0.0);
            continue;
        }

        // Thumb height proportional to visible ratio
        let visible_ratio = (viewport_height / content_height).clamp(0.05, 1.0);
        let thumb_height = (track_height * visible_ratio).max(20.0);

        // Thumb position proportional to scroll offset
        let max_scroll = content_height - viewport_height;
        let scroll_ratio = if max_scroll > 0.0 {
            scroll_pos.offset_y / max_scroll
        } else {
            0.0
        };
        let max_thumb_top = track_height - thumb_height;
        let thumb_top = scroll_ratio * max_thumb_top;

        thumb_node.height = Val::Px(thumb_height);
        thumb_node.top = Val::Px(thumb_top);
    }
}

/// Handles scrollbar thumb dragging.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_scrollbar_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag_state: ResMut<ScrollbarDragState>,
    thumb_query: Query<(&Interaction, &ScrollbarThumb, &ChildOf)>,
    track_query: Query<&ComputedNode, With<ScrollbarTrack>>,
    mut scroll_query: Query<(&mut ScrollPosition, &ComputedNode, &Children), With<ScrollView>>,
    child_nodes: Query<&ComputedNode, (Without<ScrollView>, Without<ScrollbarTrack>)>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
) {
    let cursor_y = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .map(|p| p.y)
        .unwrap_or(0.0);

    // Start drag
    if mouse.just_pressed(MouseButton::Left) && drag_state.dragging.is_none() {
        for (interaction, thumb, child_of) in &thumb_query {
            if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
                let scroll_entity = thumb.scroll_view;

                if let Some(ref scope) = scope {
                    if !is_in_scope(scroll_entity, scope, &parents) {
                        continue;
                    }
                }

                let Ok((scroll_pos, viewport_node, children)) = scroll_query.get(scroll_entity)
                else {
                    continue;
                };

                let Ok(track_node) = track_query.get(child_of.parent()) else {
                    continue;
                };

                // Calculate content height and max scroll
                let viewport_height = viewport_node.size().y;
                let mut content_height: f32 = 0.0;
                for child in children.iter() {
                    if let Ok(child_node) = child_nodes.get(child) {
                        content_height += child_node.size().y;
                    }
                }
                let max_scroll = (content_height - viewport_height).max(0.0);
                let track_height = track_node.size().y;
                let visible_ratio = (viewport_height / content_height).clamp(0.05, 1.0);
                let thumb_height = (track_height * visible_ratio).max(20.0);
                let usable_track = (track_height - thumb_height).max(1.0);

                drag_state.dragging = Some(scroll_entity);
                drag_state.start_mouse_y = cursor_y;
                drag_state.start_scroll_offset = scroll_pos.offset_y;
                drag_state.max_scroll = max_scroll;
                drag_state.usable_track = usable_track;
                break;
            }
        }
    }

    // Continue drag
    if let Some(scroll_entity) = drag_state.dragging {
        if mouse.pressed(MouseButton::Left) {
            let delta_mouse = cursor_y - drag_state.start_mouse_y;

            if drag_state.usable_track > 0.0 && drag_state.max_scroll > 0.0 {
                // Convert pixel mouse delta to scroll delta (1:1 with thumb movement)
                let scroll_delta = delta_mouse * (drag_state.max_scroll / drag_state.usable_track);
                let new_offset =
                    (drag_state.start_scroll_offset + scroll_delta).clamp(0.0, drag_state.max_scroll);

                if let Ok((mut scroll_pos, _, _)) = scroll_query.get_mut(scroll_entity) {
                    scroll_pos.offset_y = new_offset;
                }
            }
        } else {
            // Mouse released
            drag_state.dragging = None;
        }
    }
}

/// Handles clicking on the scrollbar track (page up/down).
/// Clicking above the thumb scrolls up by one viewport, below scrolls down.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn handle_track_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    drag_state: Res<ScrollbarDragState>,
    track_query: Query<
        (Entity, &Interaction, &ScrollbarTrack, &GlobalTransform, &ComputedNode),
        Without<ScrollView>,
    >,
    mut scroll_query: Query<(&mut ScrollPosition, &ComputedNode, &Children), With<ScrollView>>,
    child_nodes: Query<&ComputedNode, (Without<ScrollView>, Without<ScrollbarTrack>)>,
    scope: Option<Res<UiInputScope>>,
    scope_parents: Query<&ChildOf>,
) {
    // Don't handle track clicks while thumb is being dragged
    if drag_state.dragging.is_some() {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_y) = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .map(|p| p.y)
    else {
        return;
    };

    for (entity, interaction, track, track_transform, track_node) in &track_query {
        if *interaction != Interaction::Pressed && *interaction != Interaction::Hovered {
            continue;
        }

        if let Some(ref scope) = scope {
            if !is_in_scope(entity, scope, &scope_parents) {
                continue;
            }
        }

        let Ok((mut scroll_pos, viewport_node, children)) =
            scroll_query.get_mut(track.scroll_view)
        else {
            continue;
        };

        let viewport_height = viewport_node.size().y;
        let track_height = track_node.size().y;

        if track_height <= 0.0 || viewport_height <= 0.0 {
            continue;
        }

        // Content height
        let mut content_height: f32 = 0.0;
        for child in children.iter() {
            if let Ok(child_node) = child_nodes.get(child) {
                content_height += child_node.size().y;
            }
        }
        let max_scroll = (content_height - viewport_height).max(0.0);
        if max_scroll <= 0.0 {
            continue;
        }

        // Where on the track did we click? (0.0 = top, 1.0 = bottom)
        let track_top = track_transform.translation().y - track_height / 2.0;
        let click_ratio = ((cursor_y - track_top) / track_height).clamp(0.0, 1.0);

        // Where is the thumb currently? (ratio)
        let current_ratio = scroll_pos.offset_y / max_scroll;

        // Page scroll: move by viewport_height in the appropriate direction
        if click_ratio < current_ratio {
            scroll_pos.offset_y = (scroll_pos.offset_y - viewport_height).max(0.0);
        } else {
            scroll_pos.offset_y = (scroll_pos.offset_y + viewport_height).min(max_scroll);
        }

        break;
    }
}

/// Run condition: returns true when any ScrollView entities exist.
pub fn has_scroll_views(query: Query<(), With<ScrollView>>) -> bool {
    !query.is_empty()
}
