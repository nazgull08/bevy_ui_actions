use bevy::prelude::*;

/// Font size + color pair for a [`TextRole`].
///
/// # Example
///
/// ```rust
/// use bevy_ui_actions::prelude::*;
///
/// let preset = TextPreset::new(18.0, Color::WHITE);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TextPreset {
    /// Font size in pixels.
    pub size: f32,
    /// Text color.
    pub color: Color,
}

impl TextPreset {
    /// Create a new preset with the given size and color.
    pub const fn new(size: f32, color: Color) -> Self {
        Self { size, color }
    }
}

/// Global UI theme resource.
///
/// Stores font handles and typography presets for all [`TextRole`]s.
/// Modify presets to change text appearance globally.
///
/// # Example
///
/// ```rust
/// fn setup(mut theme: ResMut<UiTheme>, asset_server: Res<AssetServer>) {
///     theme.font = asset_server.load("fonts/my_font.ttf");
///     // Bump all sizes by 2px
///     theme.title.size = 38.0;
///     theme.heading.size = 20.0;
///     theme.body.size = 16.0;
///     theme.button.size = 18.0;
///     theme.label.size = 13.0;
///     theme.caption.size = 12.0;
/// }
/// ```
#[derive(Resource, Clone)]
pub struct UiTheme {
    /// Primary font for all UI text.
    pub font: Handle<Font>,

    /// Secondary font (monospace, pixel, etc). Optional.
    pub font_alt: Option<Handle<Font>>,

    /// Title preset (e.g. "SUBRIDERE"). Default: 36px, white.
    pub title: TextPreset,
    /// Section heading preset (e.g. "Equipment"). Default: 18px, bright.
    pub heading: TextPreset,
    /// Body text preset (descriptions, dialogue). Default: 14px, light gray.
    pub body: TextPreset,
    /// Button label preset. Default: 16px, bright.
    pub button: TextPreset,
    /// Small label preset (slot names, hints). Default: 11px, dim.
    pub label: TextPreset,
    /// Tiny caption preset (footnotes). Default: 10px, dim.
    pub caption: TextPreset,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            font: Handle::default(),
            font_alt: None,
            title: TextPreset::new(36.0, Color::srgb(0.95, 0.95, 0.95)),
            heading: TextPreset::new(18.0, Color::srgb(0.9, 0.9, 0.9)),
            body: TextPreset::new(14.0, Color::srgb(0.75, 0.75, 0.75)),
            button: TextPreset::new(16.0, Color::srgb(0.85, 0.85, 0.85)),
            label: TextPreset::new(11.0, Color::srgb(0.5, 0.5, 0.55)),
            caption: TextPreset::new(10.0, Color::srgb(0.4, 0.4, 0.45)),
        }
    }
}

impl UiTheme {
    /// Look up the preset for a given role.
    pub fn preset(&self, role: TextRole) -> &TextPreset {
        match role {
            TextRole::Title => &self.title,
            TextRole::Heading => &self.heading,
            TextRole::Body => &self.body,
            TextRole::Button => &self.button,
            TextRole::Label => &self.label,
            TextRole::Caption => &self.caption,
        }
    }

    /// Create a theme with all sizes scaled by a factor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy_ui_actions::prelude::UiTheme;
    ///
    /// let theme = UiTheme::default().with_scale(1.2);
    /// assert!((theme.body.size - 16.8).abs() < 0.01);
    /// ```
    pub fn with_scale(mut self, factor: f32) -> Self {
        self.title.size *= factor;
        self.heading.size *= factor;
        self.body.size *= factor;
        self.button.size *= factor;
        self.label.size *= factor;
        self.caption.size *= factor;
        self
    }

    /// Create a theme with a flat offset added to all sizes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy_ui_actions::prelude::UiTheme;
    ///
    /// let theme = UiTheme::default().with_offset(2.0);
    /// assert!((theme.body.size - 16.0).abs() < 0.01);
    /// ```
    pub fn with_offset(mut self, offset: f32) -> Self {
        self.title.size += offset;
        self.heading.size += offset;
        self.body.size += offset;
        self.button.size += offset;
        self.label.size += offset;
        self.caption.size += offset;
        self
    }
}

/// Semantic text role with default size and color.
///
/// Use roles for consistent typography across your UI.
/// Each role maps to a [`TextPreset`] in [`UiTheme`].
/// The static [`TextRole::size`] and [`TextRole::color`] methods return
/// built-in defaults; the actual values used at runtime come from the theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum TextRole {
    /// Large title (e.g. "SUBRIDERE"). Default: 36px, white.
    Title,
    /// Section heading (e.g. "Equipment"). Default: 18px, bright.
    Heading,
    /// Body text (descriptions, dialogue). Default: 14px, light gray.
    Body,
    /// Button label. Default: 16px, bright.
    Button,
    /// Small label (slot names, hints). Default: 11px, dim.
    Label,
    /// Tiny caption (footnotes). Default: 10px, dim.
    Caption,
}

impl TextRole {
    /// Built-in default font size for this role.
    ///
    /// Prefer reading from [`UiTheme::preset`] at runtime.
    pub fn size(self) -> f32 {
        match self {
            Self::Title => 36.0,
            Self::Heading => 18.0,
            Self::Body => 14.0,
            Self::Button => 16.0,
            Self::Label => 11.0,
            Self::Caption => 10.0,
        }
    }

    /// Built-in default text color for this role.
    ///
    /// Prefer reading from [`UiTheme::preset`] at runtime.
    pub fn color(self) -> Color {
        match self {
            Self::Title => Color::srgb(0.95, 0.95, 0.95),
            Self::Heading => Color::srgb(0.9, 0.9, 0.9),
            Self::Body => Color::srgb(0.75, 0.75, 0.75),
            Self::Button => Color::srgb(0.85, 0.85, 0.85),
            Self::Label => Color::srgb(0.5, 0.5, 0.55),
            Self::Caption => Color::srgb(0.4, 0.4, 0.45),
        }
    }
}

/// Extension trait for spawning text using [`UiTheme`].
///
/// All methods read the font from the global `UiTheme` resource.
/// Returns `EntityCommands` so you can chain `.insert()` calls.
///
/// # Example
///
/// ```rust
/// fn setup(mut commands: Commands) {
///     commands.spawn(Node::default()).with_children(|parent| {
///         parent.ui_text(TextRole::Heading, "Equipment");
///         parent.ui_text_styled("custom", 22.0, Color::RED);
///     });
/// }
/// ```
pub trait UiTextExt {
    /// Spawn text with a semantic role (size and color from [`UiTheme`]).
    fn ui_text(&mut self, role: TextRole, text: impl Into<String>) -> EntityCommands<'_>;

    /// Spawn text with explicit size, color from role defaults (Body).
    fn ui_text_sized(&mut self, text: impl Into<String>, size: f32) -> EntityCommands<'_>;

    /// Spawn text with explicit size and color.
    fn ui_text_styled(
        &mut self,
        text: impl Into<String>,
        size: f32,
        color: Color,
    ) -> EntityCommands<'_>;
}

impl UiTextExt for ChildSpawnerCommands<'_> {
    fn ui_text(&mut self, role: TextRole, text: impl Into<String>) -> EntityCommands<'_> {
        // Spawn with placeholder size/color; resolve_ui_theme will apply
        // the actual preset from UiTheme when it sees the TextRole marker.
        self.spawn((
            Text::new(text.into()),
            TextFont::default(),
            TextColor::default(),
            role,
            UiThemedText,
        ))
    }

    fn ui_text_sized(&mut self, text: impl Into<String>, size: f32) -> EntityCommands<'_> {
        self.ui_text_styled(text, size, TextRole::Body.color())
    }

    fn ui_text_styled(
        &mut self,
        text: impl Into<String>,
        size: f32,
        color: Color,
    ) -> EntityCommands<'_> {
        // No TextRole marker — explicit size/color are used as-is.
        // resolve_ui_theme only applies font.
        self.spawn((
            Text::new(text.into()),
            TextFont {
                font_size: size,
                ..default()
            },
            TextColor(color),
            UiThemedText,
        ))
    }
}

/// Marker component for text spawned via [`UiTextExt`].
///
/// The `resolve_ui_theme` system applies the font from [`UiTheme`]
/// to all entities with this marker, then removes it.
/// If a [`TextRole`] component is also present, the system applies
/// the corresponding preset's size and color from the theme.
#[derive(Component)]
pub struct UiThemedText;

/// System: applies font (and role presets) from [`UiTheme`] to newly spawned themed text.
pub(crate) fn resolve_ui_theme(
    theme: Res<UiTheme>,
    mut query: Query<
        (Entity, &mut TextFont, &mut TextColor, Option<&TextRole>),
        With<UiThemedText>,
    >,
    mut commands: Commands,
) {
    for (entity, mut text_font, mut text_color, role) in &mut query {
        text_font.font = theme.font.clone();

        // If spawned via ui_text(role), apply preset from theme
        if let Some(&role) = role {
            let preset = theme.preset(role);
            text_font.font_size = preset.size;
            text_color.0 = preset.color;
        }

        commands.entity(entity).remove::<UiThemedText>();
    }
}
