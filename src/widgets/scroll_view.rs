use crate::core::{UiInputScope, is_in_scope};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

// PORT-0.17: ScrollPosition is a newtype over Vec2 now (`.0.x` / `.0.y`),
// and layout no longer writes ScrollPosition back — the effective offset
// lives in ComputedNode::scroll_position. Our own clamp logic still stands.

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
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
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
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .id();

        // Scroll container (takes remaining space)
        let (overflow, direction) = build_overflow(config.direction);
        let scroll_node = Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            min_width: Val::Px(0.0),
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
                    flex_shrink: 0.0,
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
                    border_radius: BorderRadius::all(Val::Px(scrollbar_width / 2.0)),
                    ..default()
                },
                BackgroundColor(thumb_color),
                Interaction::None,
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
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
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
                min_width: Val::Px(0.0),
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
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(0.0),
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
            self.entity(scroll_entity).insert(BackgroundColor(color));
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
                    flex_shrink: 0.0,
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
                    border_radius: BorderRadius::all(Val::Px(scrollbar_width / 2.0)),
                    ..default()
                },
                BackgroundColor(thumb_color),
                Interaction::None,
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
    mut wheel_events: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut query: Query<(
        Entity,
        &ScrollView,
        &mut ScrollPosition,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
) {
    let mut total_x: f32 = 0.0;
    let mut total_y: f32 = 0.0;
    let mut pixel_delta = Vec2::ZERO;

    for event in wheel_events.read() {
        let (dx, dy) = match event.unit {
            MouseScrollUnit::Line => (event.x, event.y),
            MouseScrollUnit::Pixel => {
                pixel_delta += Vec2::new(event.x, event.y);
                (0.0, 0.0)
            }
        };
        total_x += dx;
        total_y += dy;
    }

    if total_x == 0.0 && total_y == 0.0 && pixel_delta == Vec2::ZERO {
        return;
    }

    let Some(cursor) = windows
        .single()
        .ok()
        .and_then(|w| w.physical_cursor_position())
    else {
        return;
    };

    for (entity, scroll_view, mut scroll_pos, transform, computed) in &mut query {
        if !cursor_in_node(cursor, transform, computed) {
            continue;
        }

        if let Some(ref scope) = scope
            && !is_in_scope(entity, scope, &parents)
        {
            continue;
        }

        let speed = scroll_view.scroll_speed;
        let delta = Vec2::new(total_x, total_y) * speed
            + pixel_delta * computed.inverse_scale_factor() * (speed / 40.0);

        match scroll_view.direction {
            ScrollDirection::Vertical => {
                scroll_pos.0.y -= delta.y;
            }
            ScrollDirection::Horizontal => {
                scroll_pos.0.x -= delta.x;
            }
            ScrollDirection::Both => {
                scroll_pos.0.x -= delta.x;
                scroll_pos.0.y -= delta.y;
            }
        }

        // Only scroll one container (topmost under cursor)
        break;
    }
}

/// Check if cursor position falls within a UI node's bounds.
fn cursor_in_node(cursor: Vec2, transform: &UiGlobalTransform, computed: &ComputedNode) -> bool {
    let node_pos = transform.translation;
    let size = computed.size();
    let half = size / 2.0;
    cursor.x >= node_pos.x - half.x
        && cursor.x <= node_pos.x + half.x
        && cursor.y >= node_pos.y - half.y
        && cursor.y <= node_pos.y + half.y
}

/// Latch that pins a [`ScrollView`] to its bottom for a few frames.
///
/// Appending content grows the scroll body, but the new child's layout is not
/// computed until a later frame — so a one-shot `offset_y = f32::MAX` gets clamped
/// to the *old* content height and the view never reaches the new bottom. This
/// latch makes [`clamp_scroll_bounds`] re-pin to the freshly-measured bottom every
/// frame until layout settles, then removes itself.
///
/// Insert it on the scroll-view entity after appending content (see
/// `handle_topic_container`). It auto-expires, so it never locks manual scrolling.
#[derive(Component, Debug, Clone, Copy)]
pub struct StickToBottom {
    /// Frames left to keep pinning (covers multi-frame layout settling).
    pub frames: u8,
}

impl Default for StickToBottom {
    fn default() -> Self {
        Self { frames: 3 }
    }
}

/// Layout metrics in the logical units used by `ScrollPosition` and `Val::Px`.
/// Use Bevy's own layout extent rather than summing children: flex gaps, padding,
/// margins, wrapping, grids and nested content are already included by layout.
#[derive(Clone, Copy)]
struct ScrollMetrics {
    viewport: Vec2,
    max_scroll: Vec2,
}

impl ScrollMetrics {
    fn from_node(node: &ComputedNode) -> Option<Self> {
        if node.is_empty() {
            return None;
        }
        let inverse_scale = node.inverse_scale_factor();
        // This is the same physical offset bound used by Bevy's ui_layout_system.
        let viewport = (node.size() - node.scrollbar_size).max(Vec2::ZERO);
        Some(Self {
            viewport: viewport * inverse_scale,
            max_scroll: (node.content_size() - viewport).max(Vec2::ZERO) * inverse_scale,
        })
    }

    fn thumb(self, track_height: f32, offset: f32) -> Option<(f32, f32)> {
        if self.max_scroll.y <= 0.0 || track_height <= 0.0 {
            return None;
        }
        let content_height = self.viewport.y + self.max_scroll.y;
        let visible_ratio = (self.viewport.y / content_height).clamp(0.05, 1.0);
        let height = (track_height * visible_ratio).max(20.0).min(track_height);
        let top = (offset / self.max_scroll.y).clamp(0.0, 1.0) * (track_height - height);
        Some((height, top))
    }
}

/// Clamp both axes using the same extent as layout and scrollbar consumers.
/// Hidden nodes preserve their offset and pending StickToBottom latch.
pub(crate) fn clamp_scroll_bounds(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut ScrollPosition,
            &ComputedNode,
            Option<&mut StickToBottom>,
        ),
        With<ScrollView>,
    >,
) {
    for (entity, mut scroll_pos, node, stick) in &mut query {
        let Some(metrics) = ScrollMetrics::from_node(node) else {
            continue;
        };
        scroll_pos.0.x = scroll_pos.0.x.clamp(0.0, metrics.max_scroll.x);
        if let Some(mut stick) = stick {
            scroll_pos.0.y = metrics.max_scroll.y;
            if stick.frames <= 1 {
                commands.entity(entity).remove::<StickToBottom>();
            } else {
                stick.frames -= 1;
            }
        } else {
            scroll_pos.0.y = scroll_pos.0.y.clamp(0.0, metrics.max_scroll.y);
        }
    }
}

/// Updates the vertical scrollbar from the same layout metrics as clamping.
pub(crate) fn update_scrollbar_thumb(
    scroll_query: Query<(&ScrollPosition, &ComputedNode), With<ScrollView>>,
    track_query: Query<&ComputedNode, With<ScrollbarTrack>>,
    mut thumb_query: Query<(&ScrollbarThumb, &mut Node, &ChildOf)>,
) {
    for (thumb, mut thumb_node, child_of) in &mut thumb_query {
        let Ok((scroll_pos, viewport_node)) = scroll_query.get(thumb.scroll_view) else {
            continue;
        };
        let Ok(track_node) = track_query.get(child_of.parent()) else {
            continue;
        };
        let Some(metrics) = ScrollMetrics::from_node(viewport_node) else {
            continue;
        };
        let track_height = track_node.size().y * track_node.inverse_scale_factor();
        if track_height <= 0.0 {
            continue;
        }
        let (height, top) = metrics
            .thumb(track_height, scroll_pos.0.y)
            .unwrap_or((0.0, 0.0));
        thumb_node.height = Val::Px(height);
        thumb_node.top = Val::Px(top);
    }
}

/// Handles vertical scrollbar thumb dragging in logical UI coordinates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_scrollbar_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag_state: ResMut<ScrollbarDragState>,
    mut drag_scale: Local<Option<f32>>,
    thumb_query: Query<(&Interaction, &ScrollbarThumb, &ChildOf)>,
    track_query: Query<&ComputedNode, With<ScrollbarTrack>>,
    mut scroll_query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollView>>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
) {
    let Some(cursor) = windows
        .single()
        .ok()
        .filter(|w| w.focused)
        .and_then(|w| w.physical_cursor_position())
    else {
        drag_state.dragging = None;
        return;
    };
    if mouse.just_pressed(MouseButton::Left) && drag_state.dragging.is_none() {
        for (interaction, thumb, child_of) in &thumb_query {
            if *interaction != Interaction::Pressed && *interaction != Interaction::Hovered {
                continue;
            }
            let entity = thumb.scroll_view;
            if let Some(ref scope) = scope
                && !is_in_scope(entity, scope, &parents)
            {
                continue;
            }
            let Ok((scroll_pos, node)) = scroll_query.get(entity) else {
                continue;
            };
            let Some(metrics) = ScrollMetrics::from_node(node) else {
                continue;
            };
            let Ok(track) = track_query.get(child_of.parent()) else {
                continue;
            };
            let track_height = track.size().y * track.inverse_scale_factor();
            let Some((thumb_height, _)) = metrics.thumb(track_height, scroll_pos.0.y) else {
                continue;
            };
            let usable_track = track_height - thumb_height;
            if usable_track <= 0.0 {
                continue;
            }
            *drag_scale = Some(node.inverse_scale_factor());
            drag_state.dragging = Some(entity);
            drag_state.start_mouse_y = cursor.y * node.inverse_scale_factor();
            drag_state.start_scroll_offset = scroll_pos.0.y;
            drag_state.max_scroll = metrics.max_scroll.y;
            drag_state.usable_track = usable_track;
            break;
        }
    }
    if let Some(entity) = drag_state.dragging {
        if !mouse.pressed(MouseButton::Left) {
            drag_state.dragging = None;
            return;
        }
        if let Some(ref scope) = scope
            && !is_in_scope(entity, scope, &parents)
        {
            drag_state.dragging = None;
            return;
        }
        let Ok((mut scroll_pos, node)) = scroll_query.get_mut(entity) else {
            drag_state.dragging = None;
            return;
        };
        let Some(metrics) = ScrollMetrics::from_node(node) else {
            drag_state.dragging = None;
            return;
        };
        let cursor_y = cursor.y * node.inverse_scale_factor();
        let Some((_, _, child_of)) = thumb_query
            .iter()
            .find(|(_, thumb, _)| thumb.scroll_view == entity)
        else {
            drag_state.dragging = None;
            return;
        };
        let Ok(track) = track_query.get(child_of.parent()) else {
            drag_state.dragging = None;
            return;
        };
        let track_height = track.size().y * track.inverse_scale_factor();
        let Some((height, _)) = metrics.thumb(track_height, scroll_pos.0.y) else {
            drag_state.dragging = None;
            return;
        };
        let usable_track = track_height - height;
        // Rebase a held drag when resizing/content updates change its mapping.
        if *drag_scale != Some(node.inverse_scale_factor())
            || (usable_track - drag_state.usable_track).abs() > 0.01
            || (metrics.max_scroll.y - drag_state.max_scroll).abs() > 0.01
        {
            *drag_scale = Some(node.inverse_scale_factor());
            drag_state.start_mouse_y = cursor_y;
            drag_state.start_scroll_offset = scroll_pos.0.y.clamp(0.0, metrics.max_scroll.y);
            drag_state.max_scroll = metrics.max_scroll.y;
            drag_state.usable_track = usable_track;
        }
        if drag_state.usable_track > 0.0 && drag_state.max_scroll > 0.0 {
            let delta = (cursor_y - drag_state.start_mouse_y)
                * (drag_state.max_scroll / drag_state.usable_track);
            scroll_pos.0.y =
                (drag_state.start_scroll_offset + delta).clamp(0.0, metrics.max_scroll.y);
        }
    }
}

/// Page above/below the actual thumb, never when clicking inside its bounds.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn handle_track_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    drag_state: Res<ScrollbarDragState>,
    track_query: Query<
        (
            Entity,
            &Interaction,
            &ScrollbarTrack,
            &UiGlobalTransform,
            &ComputedNode,
        ),
        Without<ScrollView>,
    >,
    mut scroll_query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollView>>,
    scope: Option<Res<UiInputScope>>,
    scope_parents: Query<&ChildOf>,
) {
    if drag_state.dragging.is_some() || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = windows
        .single()
        .ok()
        .and_then(|w| w.physical_cursor_position())
    else {
        return;
    };
    for (entity, interaction, track, transform, track_node) in &track_query {
        if *interaction != Interaction::Pressed && *interaction != Interaction::Hovered {
            continue;
        }
        if let Some(ref scope) = scope
            && !is_in_scope(entity, scope, &scope_parents)
        {
            continue;
        }
        let Ok((mut scroll_pos, node)) = scroll_query.get_mut(track.scroll_view) else {
            continue;
        };
        let Some(metrics) = ScrollMetrics::from_node(node) else {
            continue;
        };
        let track_height = track_node.size().y * track_node.inverse_scale_factor();
        let Some((height, top)) = metrics.thumb(track_height, scroll_pos.0.y) else {
            continue;
        };
        let physical_top = transform.translation.y - track_node.size().y / 2.0;
        let click_y = (cursor.y - physical_top) * track_node.inverse_scale_factor();
        if click_y < top {
            scroll_pos.0.y = (scroll_pos.0.y - metrics.viewport.y).max(0.0);
        } else if click_y > top + height {
            scroll_pos.0.y = (scroll_pos.0.y + metrics.viewport.y).min(metrics.max_scroll.y);
        }
        break;
    }
}

/// Run condition: returns true when any ScrollView entities exist.
pub fn has_scroll_views(query: Query<(), With<ScrollView>>) -> bool {
    !query.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_extent_matches_scaled_engine_scroll_bound() {
        let node = ComputedNode {
            size: Vec2::new(300.0, 180.0),
            content_size: Vec2::new(540.0, 750.0),
            scrollbar_size: Vec2::new(15.0, 0.0),
            inverse_scale_factor: 2.0 / 3.0,
            ..default()
        };
        let metrics = ScrollMetrics::from_node(&node).unwrap();
        assert_eq!(metrics.viewport, Vec2::new(190.0, 120.0));
        assert_eq!(metrics.max_scroll, Vec2::new(170.0, 380.0));
    }

    #[test]
    fn empty_layout_has_no_scroll_range_or_thumb() {
        let metrics = ScrollMetrics::from_node(&ComputedNode {
            size: Vec2::new(200.0, 120.0),
            content_size: Vec2::ZERO,
            inverse_scale_factor: 1.0,
            ..default()
        })
        .unwrap();
        assert_eq!(metrics.max_scroll, Vec2::ZERO);
        assert!(metrics.thumb(120.0, 20.0).is_none());
    }

    #[test]
    fn hidden_layout_skips_scroll_mutation() {
        assert!(ScrollMetrics::from_node(&ComputedNode::default()).is_none());
    }

    #[test]
    fn short_track_thumb_cannot_extend_outside_track() {
        let metrics = ScrollMetrics {
            viewport: Vec2::new(100.0, 100.0),
            max_scroll: Vec2::new(0.0, 900.0),
        };
        assert_eq!(metrics.thumb(12.0, 900.0), Some((12.0, 0.0)));
        assert_eq!(metrics.thumb(100.0, 900.0), Some((20.0, 80.0)));
    }
}
