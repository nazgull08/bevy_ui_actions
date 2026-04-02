use bevy::prelude::*;
use bevy::text::TextLayoutInfo;

use crate::core::{TextRole, UiThemedText};

// ============================================================
// Config
// ============================================================

/// Visual configuration for hypertext.
#[derive(Clone, Debug)]
pub struct HyperTextConfig {
    /// Text role for normal (non-link) text.
    pub text_role: TextRole,
    /// Color for clickable links.
    pub link_color: Color,
    /// Color for hovered links.
    pub link_hover_color: Color,
    /// Color for visited (discovered) links. If `None`, uses `link_color`.
    pub visited_link_color: Option<Color>,
    /// Container width.
    pub width: Val,
    /// Override font size. If `None`, uses `text_role.size()`.
    pub font_size: Option<f32>,
}

impl Default for HyperTextConfig {
    fn default() -> Self {
        Self {
            text_role: TextRole::Body,
            link_color: Color::srgb(0.4, 0.6, 0.9),
            link_hover_color: Color::srgb(0.6, 0.8, 1.0),
            visited_link_color: Some(Color::srgb(0.5, 0.4, 0.7)),
            width: Val::Percent(100.0),
            font_size: None,
        }
    }
}

// ============================================================
// Components
// ============================================================

/// Marker on the root Text entity of a hypertext block.
///
/// The system uses this to find entities that need glyph hit-testing.
#[derive(Component)]
pub struct HyperText {
    /// Link spans: maps `span_index` → topic key.
    /// Only link spans are stored; plain text spans are absent.
    pub link_spans: Vec<HyperLinkSpan>,
    /// Link color (from config at spawn time).
    pub link_color: Color,
    /// Link hover color (from config at spawn time).
    pub link_hover_color: Color,
    /// Color for visited (discovered) links. If `None`, uses `link_color`.
    pub visited_link_color: Option<Color>,
    /// Font size used at spawn time, needed for hit-test line height.
    pub font_size: f32,
}

/// A link within a hypertext block.
#[derive(Clone, Debug)]
pub struct HyperLinkSpan {
    /// The span index in the Text entity's children (0 = root Text, 1+ = TextSpan children).
    pub span_index: usize,
    /// The topic key from `[Display|key]` markup.
    pub topic: String,
}

/// Tracks which link span is currently hovered (if any), to avoid redundant color updates.
#[derive(Component, Default)]
pub struct HyperTextHoverState {
    /// Currently hovered span_index, or None.
    pub hovered_span: Option<usize>,
}

// ============================================================
// Event
// ============================================================

/// Fired when the user clicks a hyperlink in a [`HyperText`] block.
#[derive(Event, Debug, Clone)]
pub struct HyperLinkClicked {
    /// The topic key from `[Display|key]` markup.
    pub topic: String,
    /// The HyperText entity that was clicked.
    pub source: Entity,
}

// ============================================================
// Parser
// ============================================================

/// A parsed segment of hypertext markup.
#[derive(Clone, Debug)]
struct ParsedSegment {
    /// The display text (may start with a space).
    text: String,
    /// If this segment is a link, the topic key.
    link_topic: Option<String>,
}

/// Parse hypertext markup: `"plain text [Display|key] more text"`.
///
/// Returns a list of segments. First segment becomes the root `Text`,
/// subsequent segments become `TextSpan` children.
fn parse_hypertext(input: &str) -> Vec<ParsedSegment> {
    let mut segments = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        if let Some(bracket_start) = remaining.find('[') {
            // Plain text before the bracket
            if bracket_start > 0 {
                segments.push(ParsedSegment {
                    text: remaining[..bracket_start].to_string(),
                    link_topic: None,
                });
            }

            // Find closing bracket
            if let Some(bracket_end) = remaining[bracket_start..].find(']') {
                let bracket_end = bracket_start + bracket_end;
                let inner = &remaining[bracket_start + 1..bracket_end];

                // Split by '|' → display|key
                if let Some(pipe_pos) = inner.find('|') {
                    let display = &inner[..pipe_pos];
                    let key = &inner[pipe_pos + 1..];
                    segments.push(ParsedSegment {
                        text: display.to_string(),
                        link_topic: Some(key.to_string()),
                    });
                } else {
                    // No pipe — treat the whole thing as both display and key
                    segments.push(ParsedSegment {
                        text: inner.to_string(),
                        link_topic: Some(inner.to_string()),
                    });
                }

                remaining = &remaining[bracket_end + 1..];
            } else {
                // No closing bracket — treat rest as plain text
                segments.push(ParsedSegment {
                    text: remaining.to_string(),
                    link_topic: None,
                });
                break;
            }
        } else {
            // No more brackets — rest is plain text
            segments.push(ParsedSegment {
                text: remaining.to_string(),
                link_topic: None,
            });
            break;
        }
    }

    segments
}

// ============================================================
// Spawn
// ============================================================

/// Extension trait for spawning hypertext blocks.
pub trait SpawnHyperTextExt {
    /// Spawn a hypertext block from markup string.
    ///
    /// The text is parsed for `[Display|key]` links. Plain text renders
    /// with `config.text_role` colors, links render with `config.link_color`.
    ///
    /// Returns the root entity (has [`HyperText`] component).
    fn spawn_hypertext(&mut self, config: &HyperTextConfig, text: &str) -> EntityCommands<'_>;
}

impl SpawnHyperTextExt for ChildSpawnerCommands<'_> {
    fn spawn_hypertext(&mut self, config: &HyperTextConfig, text: &str) -> EntityCommands<'_> {
        let segments = parse_hypertext(text);
        let font_size = config.font_size.unwrap_or_else(|| config.text_role.size());
        let normal_color = config.text_role.color();
        let link_color = config.link_color;

        // Collect link span info
        let mut link_spans = Vec::new();

        // First segment → root Text entity
        let (root_text, root_color) = if let Some(first) = segments.first() {
            let color = if first.link_topic.is_some() {
                link_color
            } else {
                normal_color
            };
            if let Some(ref topic) = first.link_topic {
                link_spans.push(HyperLinkSpan {
                    span_index: 0,
                    topic: topic.clone(),
                });
            }
            (first.text.clone(), color)
        } else {
            (String::new(), normal_color)
        };

        // Collect child span data before spawning
        struct SpanData {
            text: String,
            color: Color,
        }
        let mut child_spans: Vec<SpanData> = Vec::new();

        for (i, segment) in segments.iter().enumerate().skip(1) {
            let color = if segment.link_topic.is_some() {
                link_color
            } else {
                normal_color
            };
            if let Some(ref topic) = segment.link_topic {
                link_spans.push(HyperLinkSpan {
                    span_index: i,
                    topic: topic.clone(),
                });
            }
            child_spans.push(SpanData {
                text: segment.text.clone(),
                color,
            });
        }

        let hyper = HyperText {
            link_spans,
            link_color,
            link_hover_color: config.link_hover_color,
            visited_link_color: config.visited_link_color,
            font_size,
        };

        // Spawn root Text entity
        let mut ec = self.spawn((
            Text::new(root_text),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(root_color),
            UiThemedText,
            Node {
                width: config.width,
                ..default()
            },
            hyper,
            HyperTextHoverState::default(),
        ));

        // Spawn TextSpan children
        ec.with_children(|parent| {
            for span_data in child_spans {
                parent.spawn((
                    TextSpan::new(span_data.text),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextColor(span_data.color),
                    UiThemedText,
                ));
            }
        });

        ec
    }
}

// ============================================================
// Systems
// ============================================================

/// Handles mouse clicks on hypertext links via glyph hit-testing.
pub(crate) fn hypertext_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    query: Query<(Entity, &HyperText, &TextLayoutInfo, &GlobalTransform, &ComputedNode)>,
    mut events: EventWriter<HyperLinkClicked>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.single().ok().and_then(|w| w.cursor_position()) else {
        return;
    };

    for (entity, hyper, layout, transform, computed) in &query {
        let Some(topic) = hit_test_link(cursor, hyper, layout, transform, computed) else {
            continue;
        };

        events.write(HyperLinkClicked {
            topic,
            source: entity,
        });
        break;
    }
}

/// Updates link text color on hover via glyph hit-testing.
///
/// When a `TopicRegistry` resource exists and `visited_link_color` is set,
/// links for discovered topics show in the visited color instead of the default link color.
pub(crate) fn hypertext_hover(
    windows: Query<&Window>,
    mut query: Query<(
        &HyperText,
        &TextLayoutInfo,
        &GlobalTransform,
        &ComputedNode,
        &mut HyperTextHoverState,
        &Children,
    )>,
    mut span_colors: Query<&mut TextColor>,
    registry: Option<Res<crate::widgets::dialogue::TopicRegistry>>,
) {
    let cursor = windows.single().ok().and_then(|w| w.cursor_position());

    for (hyper, layout, transform, computed, mut hover_state, children) in &mut query {
        let hovered_span = cursor
            .and_then(|c| hit_test_span_index(c, hyper, layout, transform, computed));

        if hovered_span == hover_state.hovered_span {
            continue;
        }

        let old = hover_state.hovered_span;
        hover_state.hovered_span = hovered_span;

        // Restore old hovered span to link_color (or visited_link_color)
        if let Some(old_idx) = old {
            if let Some(link) = hyper.link_spans.iter().find(|l| l.span_index == old_idx) {
                let color = resolve_link_color(hyper, &link.topic, registry.as_deref());
                set_span_color(old_idx, color, children, &mut span_colors);
            }
        }

        // Set new hovered span to hover_color
        if let Some(new_idx) = hovered_span {
            set_span_color(new_idx, hyper.link_hover_color, children, &mut span_colors);
        }
    }
}

/// Resolve the resting color for a link: visited color if topic is discovered, else default link color.
fn resolve_link_color(
    hyper: &HyperText,
    topic: &str,
    registry: Option<&crate::widgets::dialogue::TopicRegistry>,
) -> Color {
    if let Some(visited_color) = hyper.visited_link_color {
        if let Some(reg) = registry {
            if reg.is_discovered(topic) {
                return visited_color;
            }
        }
    }
    hyper.link_color
}

/// Updates link colors to visited_link_color when topics become discovered.
///
/// Runs after `handle_dialogue_topic` discovers topics, so that all existing
/// hypertext blocks (including the one that was just clicked) get updated.
pub(crate) fn update_visited_link_colors(
    mut discovered_events: EventReader<crate::widgets::dialogue::TopicDiscovered>,
    query: Query<(&HyperText, &HyperTextHoverState, &Children)>,
    mut span_colors: Query<&mut TextColor>,
    registry: Option<Res<crate::widgets::dialogue::TopicRegistry>>,
) {
    let events: Vec<_> = discovered_events.read().collect();
    if events.is_empty() {
        return;
    }

    let Some(registry) = registry else { return };

    // For each discovered topic, find all HyperText entities with matching links
    for event in &events {
        for (hyper, hover_state, children) in &query {
            let Some(visited_color) = hyper.visited_link_color else {
                continue;
            };

            for link in &hyper.link_spans {
                if link.topic != event.topic {
                    continue;
                }
                if !registry.is_discovered(&link.topic) {
                    continue;
                }
                // Don't override hover color
                if hover_state.hovered_span == Some(link.span_index) {
                    continue;
                }
                set_span_color(link.span_index, visited_color, children, &mut span_colors);
            }
        }
    }
}

/// Applies visited_link_color to newly spawned HyperText entities
/// for links whose topics are already discovered in TopicRegistry.
pub(crate) fn apply_initial_visited_colors(
    query: Query<(&HyperText, &HyperTextHoverState, &Children), Added<HyperText>>,
    mut span_colors: Query<&mut TextColor>,
    registry: Option<Res<crate::widgets::dialogue::TopicRegistry>>,
) {
    let Some(registry) = registry else { return };

    for (hyper, hover_state, children) in &query {
        let Some(visited_color) = hyper.visited_link_color else {
            continue;
        };

        for link in &hyper.link_spans {
            if !registry.is_discovered(&link.topic) {
                continue;
            }
            if hover_state.hovered_span == Some(link.span_index) {
                continue;
            }
            set_span_color(link.span_index, visited_color, children, &mut span_colors);
        }
    }
}

/// Run condition: true when any HyperText entities exist.
pub fn has_hypertext(query: Query<(), With<HyperText>>) -> bool {
    !query.is_empty()
}

// ============================================================
// Hit-testing helpers
// ============================================================

/// A bounding rectangle for a span on a single line.
struct SpanRect {
    span_index: usize,
    min: Vec2,
    max: Vec2,
}

/// Build bounding rects for all link spans.
///
/// A span that wraps across two visual rows produces two `SpanRect`s.
/// Horizontal extent: first glyph min to last glyph max on that row.
/// Vertical extent: span's glyph y-center ± font_size (covers full line height
/// regardless of glyph case). We cluster glyphs into visual rows by y-proximity
/// because `glyph.line_index` is unreliable (always 0 for wrapped text).
fn build_span_rects(
    layout: &TextLayoutInfo,
    link_span_indices: &[usize],
    font_size: f32,
) -> Vec<SpanRect> {
    use std::collections::HashMap;

    // Collect per (span_index, visual_row) → (min_x, max_x, y_sum, y_count)
    // Visual row is determined by clustering glyph y-centers.
    let half_line = font_size * 0.6; // generous half-line-height for AABB

    // First: find all visual row y-centers from ALL glyphs (not just links).
    let mut row_centers: Vec<f32> = Vec::new();
    for glyph in &layout.glyphs {
        if glyph.size.x == 0.0 && glyph.size.y == 0.0 {
            continue; // skip zero-size glyphs (spaces, etc.)
        }
        let cy = glyph.position.y + glyph.size.y * 0.5;
        let found = row_centers.iter().any(|&rc| (rc - cy).abs() < font_size * 0.5);
        if !found {
            row_centers.push(cy);
        }
    }
    row_centers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Helper: find which visual row a glyph belongs to.
    let find_row = |cy: f32| -> usize {
        row_centers
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - cy).abs().partial_cmp(&(*b - cy).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    };

    // Collect link span extents per (span_index, visual_row).
    let mut map: HashMap<(usize, usize), (f32, f32, f32)> = HashMap::new(); // (min_x, max_x, row_center_y)

    for glyph in &layout.glyphs {
        if !link_span_indices.contains(&glyph.span_index) {
            continue;
        }
        if glyph.size.x == 0.0 && glyph.size.y == 0.0 {
            continue;
        }

        let cy = glyph.position.y + glyph.size.y * 0.5;
        let row = find_row(cy);
        let row_cy = row_centers.get(row).copied().unwrap_or(cy);

        let key = (glyph.span_index, row);
        map.entry(key)
            .and_modify(|(min_x, max_x, _)| {
                *min_x = min_x.min(glyph.position.x);
                *max_x = max_x.max(glyph.position.x + glyph.size.x);
            })
            .or_insert((glyph.position.x, glyph.position.x + glyph.size.x, row_cy));
    }

    map.into_iter()
        .map(|((span_index, _row), (min_x, max_x, row_cy))| SpanRect {
            span_index,
            min: Vec2::new(min_x, row_cy - half_line),
            max: Vec2::new(max_x, row_cy + half_line),
        })
        .collect()
}

/// Convert screen-space cursor to local text node space.
/// Returns `None` if cursor is outside the node bounds.
fn cursor_to_local(
    cursor: Vec2,
    transform: &GlobalTransform,
    computed: &ComputedNode,
) -> Option<Vec2> {
    let node_pos = transform.translation().truncate();
    let node_size = computed.size();
    let half = node_size / 2.0;

    if cursor.x < node_pos.x - half.x
        || cursor.x > node_pos.x + half.x
        || cursor.y < node_pos.y - half.y
        || cursor.y > node_pos.y + half.y
    {
        return None;
    }

    Some(Vec2::new(
        cursor.x - (node_pos.x - half.x),
        cursor.y - (node_pos.y - half.y),
    ))
}

/// Hit-test a cursor position against link span bounding boxes.
/// Returns the topic key if a link was hit.
fn hit_test_link(
    cursor: Vec2,
    hyper: &HyperText,
    layout: &TextLayoutInfo,
    transform: &GlobalTransform,
    computed: &ComputedNode,
) -> Option<String> {
    let span_index = hit_test_span_index(cursor, hyper, layout, transform, computed)?;
    hyper
        .link_spans
        .iter()
        .find(|l| l.span_index == span_index)
        .map(|l| l.topic.clone())
}

/// Hit-test a cursor position against link span bounding boxes.
/// Returns the span_index of the link under the cursor (if any).
fn hit_test_span_index(
    cursor: Vec2,
    hyper: &HyperText,
    layout: &TextLayoutInfo,
    transform: &GlobalTransform,
    computed: &ComputedNode,
) -> Option<usize> {
    let local = cursor_to_local(cursor, transform, computed)?;

    let link_span_indices: Vec<usize> = hyper.link_spans.iter().map(|l| l.span_index).collect();
    let rects = build_span_rects(layout, &link_span_indices, hyper.font_size);

    for rect in &rects {
        if local.x >= rect.min.x
            && local.x <= rect.max.x
            && local.y >= rect.min.y
            && local.y <= rect.max.y
        {
            return Some(rect.span_index);
        }
    }

    None
}

// ============================================================
// Color helpers
// ============================================================


/// Set the TextColor of a span by index.
///
/// span_index 0 = root Text entity (not in children).
/// span_index N = children\[N-1\].
fn set_span_color(
    span_index: usize,
    color: Color,
    children: &Children,
    span_colors: &mut Query<&mut TextColor>,
) {
    if span_index == 0 {
        // Root entity — handled separately if needed.
        // For now, links in root span position are rare.
        return;
    }

    let child_index = span_index - 1;
    if let Some(&child_entity) = children.iter().collect::<Vec<_>>().get(child_index) {
        if let Ok(mut text_color) = span_colors.get_mut(child_entity) {
            text_color.0 = color;
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text() {
        let segments = parse_hypertext("Hello world");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");
        assert!(segments[0].link_topic.is_none());
    }

    #[test]
    fn parse_single_link() {
        let segments = parse_hypertext("Talk about [Nerevarine|nerevarine] prophecy");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "Talk about ");
        assert!(segments[0].link_topic.is_none());
        assert_eq!(segments[1].text, "Nerevarine");
        assert_eq!(segments[1].link_topic.as_deref(), Some("nerevarine"));
        assert_eq!(segments[2].text, " prophecy");
        assert!(segments[2].link_topic.is_none());
    }

    #[test]
    fn parse_multiple_links() {
        let segments = parse_hypertext("[Vivec|vivec] and [Dagoth Ur|dagoth_ur]");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "Vivec");
        assert_eq!(segments[0].link_topic.as_deref(), Some("vivec"));
        assert_eq!(segments[1].text, " and ");
        assert!(segments[1].link_topic.is_none());
        assert_eq!(segments[2].text, "Dagoth Ur");
        assert_eq!(segments[2].link_topic.as_deref(), Some("dagoth_ur"));
    }

    #[test]
    fn parse_no_pipe_uses_text_as_key() {
        let segments = parse_hypertext("See [Blight] disease");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[1].text, "Blight");
        assert_eq!(segments[1].link_topic.as_deref(), Some("Blight"));
    }

    #[test]
    fn parse_unclosed_bracket() {
        let segments = parse_hypertext("broken [link text");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "broken [link text");
    }

    #[test]
    fn parse_empty_input() {
        let segments = parse_hypertext("");
        assert!(segments.is_empty());
    }

    #[test]
    fn parse_adjacent_links() {
        let segments = parse_hypertext("[A|a][B|b]");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].link_topic.as_deref(), Some("a"));
        assert_eq!(segments[1].link_topic.as_deref(), Some("b"));
    }
}
