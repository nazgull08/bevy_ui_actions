use bevy::prelude::*;

/// Global UI theme resource.
///
/// Stores the default font for all UI text. Set this resource after loading
/// your font asset, or leave as default to use Bevy's built-in font.
///
/// # Example
///
/// ```rust
/// fn setup(mut theme: ResMut<UiTheme>, asset_server: Res<AssetServer>) {
///     theme.font = asset_server.load("fonts/my_font.ttf");
/// }
/// ```
#[derive(Resource, Clone, Default)]
pub struct UiTheme {
    /// Primary font for all UI text.
    pub font: Handle<Font>,

    /// Secondary font (monospace, pixel, etc). Optional.
    pub font_alt: Option<Handle<Font>>,
}

/// Semantic text role with default size and color.
///
/// Use roles for consistent typography across your UI.
/// Each role maps to a (size, color) pair via [`TextRole::size`] and [`TextRole::color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextRole {
    /// Large title (e.g. "SUBRIDERE"). 36px, white.
    Title,
    /// Section heading (e.g. "Equipment"). 18px, bright.
    Heading,
    /// Body text (descriptions, dialogue). 14px, light gray.
    Body,
    /// Button label. 16px, bright.
    Button,
    /// Small label (slot names, hints). 11px, dim.
    Label,
    /// Tiny caption (footnotes). 10px, dim.
    Caption,
}

impl TextRole {
    /// Default font size for this role.
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

    /// Default text color for this role.
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
    /// Spawn text with a semantic role (size and color from [`TextRole`]).
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
        self.ui_text_styled(text, role.size(), role.color())
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
        // Read font from theme resource.
        // ChildSpawnerCommands doesn't give us resource access directly,
        // so we spawn with default font and let resolve_ui_theme fix it.
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
#[derive(Component)]
pub struct UiThemedText;

/// System: applies font from [`UiTheme`] to newly spawned themed text.
pub(crate) fn resolve_ui_theme(
    theme: Res<UiTheme>,
    mut query: Query<(Entity, &mut TextFont), With<UiThemedText>>,
    mut commands: Commands,
) {
    for (entity, mut text_font) in &mut query {
        text_font.font = theme.font.clone();
        commands.entity(entity).remove::<UiThemedText>();
    }
}
