//! Rich Tooltip widget with support for structured content.
//!
//! # Examples
//!
//! Simple text tooltip (backwards compatible):
//! ```rust
//! Tooltip::new("Click to activate")
//! ```
//!
//! Rich tooltip with builder:
//! ```rust
//! Tooltip::builder()
//!     .title("Iron Sword")
//!     .subtitle("Weapon • Main Hand")
//!     .separator()
//!     .stat("Damage", "12")
//!     .stat_diff("Speed", "1.2x", StatDiff::Better(0.2))
//!     .separator()
//!     .text("A reliable iron sword.")
//!     .build()
//! ```

use bevy::prelude::*;

use crate::DragState;

// ============================================================
// System Sets
// ============================================================

/// System sets for tooltip lifecycle. Use these to order your systems.
///
/// Order: DetectHover → GenerateContent → Display
///
/// Put your dynamic tooltip generation systems in `GenerateContent`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TooltipSet {
    /// Detect hover state over UI elements with Tooltip component
    DetectHover,
    /// Generate/update tooltip content (external systems run here)
    GenerateContent,
    /// Show/hide tooltip UI based on content and delay
    Display,
}

// ============================================================
// Tooltip Sections
// ============================================================

/// A single section within a rich tooltip.
#[derive(Clone, Debug)]
pub enum TooltipSection {
    /// Large title text (item name)
    Title(String),

    /// Smaller subtitle (category, slot)
    Subtitle(String),

    /// Horizontal separator line
    Separator,

    /// Stat with optional comparison diff
    Stat {
        label: String,
        value: String,
        diff: Option<StatDiff>,
    },

    /// Plain text paragraph (description)
    Text(String),

    /// Key-value pair on single line
    KeyValue(String, String),

    /// Vertical spacer
    Spacer(f32),
}

/// Indicates how a stat compares to current equipment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatDiff {
    /// Better than current (green, arrow up)
    Better(f32),
    /// Worse than current (red, arrow down)
    Worse(f32),
    /// No change (gray)
    Neutral,
}

// ============================================================
// Tooltip Content
// ============================================================

/// Content of a tooltip - either simple text or structured sections.
#[derive(Clone, Debug, Default)]
pub enum TooltipContent {
    /// Empty tooltip (won't show)
    #[default]
    Empty,

    /// Simple text (backwards compatible)
    Text(String),

    /// Rich structured content
    Sections(Vec<TooltipSection>),
}

impl TooltipContent {
    pub fn is_empty(&self) -> bool {
        match self {
            TooltipContent::Empty => true,
            TooltipContent::Text(s) => s.is_empty(),
            TooltipContent::Sections(s) => s.is_empty(),
        }
    }
}

// ============================================================
// Tooltip Component
// ============================================================

/// Tooltip component - attach to any interactive UI element.
///
/// Shows a tooltip after hovering for `delay_ms` milliseconds.
#[derive(Component, Clone)]
pub struct Tooltip {
    pub content: TooltipContent,
    pub delay_ms: u32,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            content: TooltipContent::Empty,
            delay_ms: 500,
        }
    }
}

impl Tooltip {
    /// Create a simple text tooltip.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: TooltipContent::Text(text.into()),
            delay_ms: 500,
        }
    }

    /// Create a simple text tooltip with custom delay.
    pub fn with_delay(text: impl Into<String>, delay_ms: u32) -> Self {
        Self {
            content: TooltipContent::Text(text.into()),
            delay_ms,
        }
    }

    /// Create a rich tooltip from sections.
    pub fn rich(sections: Vec<TooltipSection>) -> Self {
        Self {
            content: TooltipContent::Sections(sections),
            delay_ms: 300,
        }
    }

    /// Start building a rich tooltip.
    pub fn builder() -> TooltipBuilder {
        TooltipBuilder::new()
    }

    /// Set delay (chainable).
    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }
}

// ============================================================
// Tooltip Builder
// ============================================================

/// Builder for constructing rich tooltips fluently.
#[derive(Default)]
pub struct TooltipBuilder {
    sections: Vec<TooltipSection>,
    delay_ms: u32,
}

impl TooltipBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            delay_ms: 300,
        }
    }

    /// Add a title section.
    pub fn title(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TooltipSection::Title(text.into()));
        self
    }

    /// Add a subtitle section.
    pub fn subtitle(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TooltipSection::Subtitle(text.into()));
        self
    }

    /// Add a separator line.
    pub fn separator(mut self) -> Self {
        self.sections.push(TooltipSection::Separator);
        self
    }

    /// Add a stat without comparison.
    pub fn stat(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.sections.push(TooltipSection::Stat {
            label: label.into(),
            value: value.into(),
            diff: None,
        });
        self
    }

    /// Add a stat with comparison diff.
    pub fn stat_diff(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        diff: StatDiff,
    ) -> Self {
        self.sections.push(TooltipSection::Stat {
            label: label.into(),
            value: value.into(),
            diff: Some(diff),
        });
        self
    }

    /// Add a text paragraph.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TooltipSection::Text(text.into()));
        self
    }

    /// Add a key-value pair.
    pub fn key_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.sections
            .push(TooltipSection::KeyValue(key.into(), value.into()));
        self
    }

    /// Add vertical spacing.
    pub fn spacer(mut self, height: f32) -> Self {
        self.sections.push(TooltipSection::Spacer(height));
        self
    }

    /// Set the delay before showing.
    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Build the tooltip.
    pub fn build(self) -> Tooltip {
        Tooltip {
            content: TooltipContent::Sections(self.sections),
            delay_ms: self.delay_ms,
        }
    }
}

// ============================================================
// Tooltip State (Resource)
// ============================================================

/// Global state for the tooltip system.
#[derive(Resource, Default)]
pub struct TooltipState {
    /// Currently hovered entity with Tooltip
    pub hovered: Option<Entity>,
    /// Time spent hovering
    pub hover_timer: f32,
    /// Is tooltip currently visible
    pub visible: bool,
    /// Spawned tooltip UI entity
    pub tooltip_entity: Option<Entity>,
}

impl TooltipState {
    pub fn clear(&mut self) {
        self.hovered = None;
        self.hover_timer = 0.0;
        self.visible = false;
    }
}

// ============================================================
// Tooltip UI Marker
// ============================================================

/// Marker component for the tooltip UI root.
#[derive(Component)]
pub struct TooltipUI;

// ============================================================
// Tooltip Style (Resource)
// ============================================================

/// Visual style for tooltips.
#[derive(Resource, Clone)]
pub struct TooltipStyle {
    // Font (required for text rendering in games with custom fonts)
    pub font: Option<Handle<Font>>,

    // Layout
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub padding: f32,
    pub max_width: f32,
    pub section_gap: f32,

    // Simple text (legacy)
    pub text_color: Color,
    pub text_size: f32,

    // Rich: Title
    pub title_color: Color,
    pub title_size: f32,

    // Rich: Subtitle
    pub subtitle_color: Color,
    pub subtitle_size: f32,

    // Rich: Stats
    pub label_color: Color,
    pub value_color: Color,
    pub stat_size: f32,

    // Rich: Diff indicators
    pub diff_better: Color,
    pub diff_worse: Color,
    pub diff_neutral: Color,

    // Rich: Separator
    pub separator_color: Color,
    pub separator_height: f32,

    // Rich: Key-Value
    pub key_color: Color,
    pub kv_size: f32,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            font: None,
            // Layout
            background: Color::srgba(0.08, 0.08, 0.10, 0.95),
            border_color: Color::srgba(0.3, 0.3, 0.35, 0.8),
            border_width: 1.0,
            padding: 10.0,
            max_width: 280.0,
            section_gap: 4.0,

            // Simple text
            text_color: Color::srgb(0.9, 0.9, 0.9),
            text_size: 14.0,

            // Title
            title_color: Color::srgb(1.0, 1.0, 1.0),
            title_size: 16.0,

            // Subtitle
            subtitle_color: Color::srgb(0.6, 0.6, 0.65),
            subtitle_size: 12.0,

            // Stats
            label_color: Color::srgb(0.7, 0.7, 0.7),
            value_color: Color::srgb(0.95, 0.95, 0.95),
            stat_size: 13.0,

            // Diff
            diff_better: Color::srgb(0.3, 0.85, 0.3),
            diff_worse: Color::srgb(0.9, 0.3, 0.3),
            diff_neutral: Color::srgb(0.5, 0.5, 0.5),

            // Separator
            separator_color: Color::srgba(0.4, 0.4, 0.45, 0.5),
            separator_height: 1.0,

            // Key-Value
            key_color: Color::srgb(0.5, 0.5, 0.55),
            kv_size: 11.0,
        }
    }
}

impl TooltipStyle {
    /// Create style with a specific font
    pub fn with_font(font: Handle<Font>) -> Self {
        Self {
            font: Some(font),
            ..Default::default()
        }
    }
}

// ============================================================
// Run Conditions
// ============================================================

/// Condition: should try to show tooltip
pub fn should_show_tooltip(state: Res<TooltipState>) -> bool {
    state.hovered.is_some() && !state.visible
}

/// Condition: tooltip is visible or has entity to clean up
pub fn should_hide_tooltip(state: Res<TooltipState>, drag: Res<DragState>) -> bool {
    state.tooltip_entity.is_some() && (state.hovered.is_none() || drag.is_dragging())
}

// ============================================================
// Systems
// ============================================================

/// Track hover state over elements with Tooltip.
///
/// Note: This tracks ANY entity with Tooltip component that is hovered,
/// regardless of whether content is empty. The content check happens in show_tooltip.
/// This allows external systems to populate content dynamically before display.
pub(crate) fn update_tooltip_hover(
    query: Query<(Entity, &Interaction), With<Tooltip>>,
    mut tooltip_state: ResMut<TooltipState>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let mut found_hovered: Option<Entity> = None;

    for (entity, interaction) in &query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            found_hovered = Some(entity);
            break;
        }
    }

    match found_hovered {
        Some(entity) => {
            if tooltip_state.hovered == Some(entity) {
                tooltip_state.hover_timer += time.delta_secs();
            } else {
                // New entity — despawn old tooltip if exists
                if let Some(old_entity) = tooltip_state.tooltip_entity.take() {
                    commands.entity(old_entity).despawn();
                }
                tooltip_state.hovered = Some(entity);
                tooltip_state.hover_timer = 0.0;
                tooltip_state.visible = false;
            }
        }
        None => {
            if tooltip_state.hovered.is_some() {
                tooltip_state.clear();
            }
        }
    }
}

/// Show tooltip when delay is reached.
///
/// Only shows if content is non-empty. This allows external systems
/// to populate content dynamically before the delay completes.
pub(crate) fn show_tooltip(
    query: Query<&Tooltip>,
    mut tooltip_state: ResMut<TooltipState>,
    tooltip_style: Res<TooltipStyle>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let Some(entity) = tooltip_state.hovered else {
        return;
    };

    let Ok(tooltip) = query.get(entity) else {
        return;
    };

    // Don't show empty tooltips
    if tooltip.content.is_empty() {
        return;
    }

    let delay_secs = tooltip.delay_ms as f32 / 1000.0;
    if tooltip_state.hover_timer < delay_secs {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let tooltip_entity =
        spawn_tooltip_ui(&mut commands, &tooltip.content, &tooltip_style, cursor_pos);

    tooltip_state.visible = true;
    tooltip_state.tooltip_entity = Some(tooltip_entity);
}

/// Hide tooltip when cursor leaves or during drag.
pub(crate) fn hide_tooltip(mut tooltip_state: ResMut<TooltipState>, mut commands: Commands) {
    if let Some(entity) = tooltip_state.tooltip_entity.take() {
        commands.entity(entity).despawn();
    }
    tooltip_state.visible = false;
    tooltip_state.hovered = None;
}

// ============================================================
// UI Spawning
// ============================================================

fn spawn_tooltip_ui(
    commands: &mut Commands,
    content: &TooltipContent,
    style: &TooltipStyle,
    cursor_pos: Vec2,
) -> Entity {
    commands
        .spawn((
            TooltipUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor_pos.x + 12.0),
                top: Val::Px(cursor_pos.y + 12.0),
                padding: UiRect::all(Val::Px(style.padding)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(style.section_gap),
                max_width: Val::Px(style.max_width),
                border: UiRect::all(Val::Px(style.border_width)),
                ..default()
            },
            BackgroundColor(style.background),
            BorderColor(style.border_color),
            GlobalZIndex(500),
        ))
        .with_children(|parent| {
            spawn_tooltip_content(parent, content, style);
        })
        .id()
}

fn spawn_tooltip_content(
    parent: &mut ChildSpawnerCommands,
    content: &TooltipContent,
    style: &TooltipStyle,
) {
    match content {
        TooltipContent::Empty => {}

        TooltipContent::Text(text) => {
            parent.spawn((
                Text::new(text),
                make_text_font(style, style.text_size),
                TextColor(style.text_color),
            ));
        }

        TooltipContent::Sections(sections) => {
            for section in sections {
                spawn_section(parent, section, style);
            }
        }
    }
}

/// Create TextFont with optional font from style
fn make_text_font(style: &TooltipStyle, size: f32) -> TextFont {
    TextFont {
        font: style.font.clone().unwrap_or_default(),
        font_size: size,
        ..default()
    }
}

fn spawn_section(
    parent: &mut ChildSpawnerCommands,
    section: &TooltipSection,
    style: &TooltipStyle,
) {
    match section {
        TooltipSection::Title(text) => {
            parent.spawn((
                Text::new(text),
                make_text_font(style, style.title_size),
                TextColor(style.title_color),
            ));
        }

        TooltipSection::Subtitle(text) => {
            parent.spawn((
                Text::new(text),
                make_text_font(style, style.subtitle_size),
                TextColor(style.subtitle_color),
            ));
        }

        TooltipSection::Separator => {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(style.separator_height),
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(style.separator_color),
            ));
        }

        TooltipSection::Stat { label, value, diff } => {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Text::new(label),
                        make_text_font(style, style.stat_size),
                        TextColor(style.label_color),
                    ));

                    // Value + diff container
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|value_row| {
                        // Value
                        value_row.spawn((
                            Text::new(value),
                            make_text_font(style, style.stat_size),
                            TextColor(style.value_color),
                        ));

                        // Diff indicator
                        if let Some(d) = diff {
                            let (diff_text, diff_color) = match d {
                                StatDiff::Better(v) => (format!("(+{:.0})", v), style.diff_better),
                                StatDiff::Worse(v) => (format!("(-{:.0})", v), style.diff_worse),
                                StatDiff::Neutral => ("(=)".to_string(), style.diff_neutral),
                            };

                            value_row.spawn((
                                Text::new(diff_text),
                                make_text_font(style, style.stat_size),
                                TextColor(diff_color),
                            ));
                        }
                    });
                });
        }

        TooltipSection::Text(text) => {
            parent.spawn((
                Text::new(text),
                make_text_font(style, style.text_size),
                TextColor(style.text_color),
            ));
        }

        TooltipSection::KeyValue(key, value) => {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("{}:", key)),
                        make_text_font(style, style.kv_size),
                        TextColor(style.key_color),
                    ));

                    row.spawn((
                        Text::new(value),
                        make_text_font(style, style.kv_size),
                        TextColor(style.text_color),
                    ));
                });
        }

        TooltipSection::Spacer(height) => {
            parent.spawn(Node {
                height: Val::Px(*height),
                ..default()
            });
        }
    }
}
