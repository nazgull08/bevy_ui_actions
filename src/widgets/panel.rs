use bevy::prelude::*;

/// Configuration for a panel widget.
///
/// A panel is a bordered container with background, padding, and flex layout.
/// Use presets like `PanelConfig::dark()` or customize fields directly.
///
/// # Example
///
/// ```rust
/// parent.spawn_panel(PanelConfig::dark()).with_children(|panel| {
///     panel.ui_text(TextRole::Heading, "Title");
/// });
/// ```
#[derive(Clone, Debug)]
pub struct PanelConfig {
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub padding: f32,
    pub gap: f32,
    pub direction: FlexDirection,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self::dark()
    }
}

impl PanelConfig {
    /// Dark panel — general purpose container.
    pub fn dark() -> Self {
        Self {
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Auto,
            min_height: Val::Auto,
            background: Color::srgb(0.12, 0.12, 0.15),
            border_color: Color::srgb(0.3, 0.3, 0.35),
            border_width: 1.0,
            padding: 15.0,
            gap: 10.0,
            direction: FlexDirection::Column,
        }
    }

    /// Overlay panel — slightly transparent, for popups/modals.
    pub fn overlay() -> Self {
        Self {
            background: Color::srgba(0.08, 0.08, 0.10, 0.95),
            border_color: Color::srgb(0.4, 0.4, 0.45),
            border_width: 2.0,
            padding: 20.0,
            gap: 15.0,
            ..Self::dark()
        }
    }

    /// Sidebar panel — taller, narrower.
    pub fn sidebar() -> Self {
        Self {
            width: Val::Px(250.0),
            height: Val::Percent(100.0),
            padding: 10.0,
            gap: 8.0,
            ..Self::dark()
        }
    }
}

/// Extension trait for spawning panels.
pub trait SpawnPanelExt {
    /// Spawn a panel with the given config. Returns `EntityCommands` for `.with_children()`.
    fn spawn_panel(&mut self, config: PanelConfig) -> EntityCommands<'_>;
}

impl SpawnPanelExt for ChildSpawnerCommands<'_> {
    fn spawn_panel(&mut self, config: PanelConfig) -> EntityCommands<'_> {
        self.spawn((
            Node {
                width: config.width,
                height: config.height,
                min_width: config.min_width,
                min_height: config.min_height,
                padding: UiRect::all(Val::Px(config.padding)),
                border: UiRect::all(Val::Px(config.border_width)),
                flex_direction: config.direction,
                row_gap: Val::Px(config.gap),
                column_gap: Val::Px(config.gap),
                ..default()
            },
            BackgroundColor(config.background),
            BorderColor(config.border_color),
        ))
    }
}

impl SpawnPanelExt for Commands<'_, '_> {
    fn spawn_panel(&mut self, config: PanelConfig) -> EntityCommands<'_> {
        self.spawn((
            Node {
                width: config.width,
                height: config.height,
                min_width: config.min_width,
                min_height: config.min_height,
                padding: UiRect::all(Val::Px(config.padding)),
                border: UiRect::all(Val::Px(config.border_width)),
                flex_direction: config.direction,
                row_gap: Val::Px(config.gap),
                column_gap: Val::Px(config.gap),
                ..default()
            },
            BackgroundColor(config.background),
            BorderColor(config.border_color),
        ))
    }
}
