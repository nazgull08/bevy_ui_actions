//! Visual feedback components and systems.
//!
//! Provides automatic visual feedback for UI elements based on interaction state.
//!
//! # Components
//!
//! - [`InteractiveVisual`] - Marker for background color feedback
//! - [`VisualStyle`] - Custom background colors (optional)
//! - [`BorderStyle`] - Border color feedback (opt-in)
//! - [`Active`] - Marker for "on" state (tabs, toggles)
//! - [`Selected`] - Marker for "selected for inspection" state
//! - [`Disabled`] - Marker for disabled elements
//!
//! # Example
//!
//! ```rust
//! // Slot with both background and border feedback
//! commands.spawn((
//!     Node { ... },
//!     BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
//!     BorderColor(Color::srgb(0.4, 0.4, 0.4)),
//!     InteractiveVisual,
//!     BorderStyle::slot(),
//!     Interaction::None,
//! ));
//!
//! // Mark as selected
//! commands.entity(slot).insert(Selected);
//! ```

use crate::core::ButtonStyle;
use bevy::prelude::*;

// ============================================================
// Marker Components
// ============================================================

/// Marker: enables automatic BackgroundColor feedback on hover/press.
///
/// Add this to any UI element with `Interaction` to get visual feedback.
/// Uses global `ButtonStyle` or local `VisualStyle` for colors.
#[derive(Component, Default)]
pub struct InteractiveVisual;

/// Marker: element is disabled (doesn't respond to clicks, dimmed visual).
#[derive(Component)]
pub struct Disabled;

/// Marker: element is active (selected tab, toggle on).
///
/// Use for persistent "on" states like current tab or enabled toggle.
#[derive(Component)]
pub struct Active;

/// Marker: element is selected for inspection/action.
///
/// Use for temporary selection in lists, grids, inventory slots.
/// Typically only one element is Selected at a time within a group.
#[derive(Component)]
pub struct Selected;

// ============================================================
// VisualStyle (Background)
// ============================================================

/// Custom background colors for a specific element.
///
/// If present, overrides the global `ButtonStyle`.
#[derive(Component, Clone, Debug)]
pub struct VisualStyle {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
    pub active: Option<Color>,
    pub selected: Option<Color>,
}

impl Default for VisualStyle {
    fn default() -> Self {
        Self {
            normal: Color::srgb(0.2, 0.2, 0.2),
            hovered: Color::srgb(0.3, 0.3, 0.3),
            pressed: Color::srgb(0.1, 0.1, 0.1),
            disabled: Color::srgb(0.15, 0.15, 0.15),
            active: None,
            selected: None,
        }
    }
}

impl VisualStyle {
    pub fn new(normal: Color, hovered: Color, pressed: Color, disabled: Color) -> Self {
        Self {
            normal,
            hovered,
            pressed,
            disabled,
            active: None,
            selected: None,
        }
    }

    pub fn with_active(mut self, active: Color) -> Self {
        self.active = Some(active);
        self
    }

    pub fn with_selected(mut self, selected: Color) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Style for tabs
    pub fn tab() -> Self {
        Self {
            normal: Color::srgb(0.15, 0.15, 0.18),
            hovered: Color::srgb(0.25, 0.25, 0.28),
            pressed: Color::srgb(0.2, 0.2, 0.23),
            disabled: Color::srgb(0.1, 0.1, 0.12),
            active: Some(Color::srgb(0.28, 0.28, 0.32)),
            selected: None,
        }
    }

    /// Style for inventory slots
    pub fn slot() -> Self {
        Self {
            normal: Color::srgb(0.15, 0.15, 0.18),
            hovered: Color::srgb(0.22, 0.22, 0.25),
            pressed: Color::srgb(0.18, 0.18, 0.20),
            disabled: Color::srgb(0.1, 0.1, 0.12),
            active: Some(Color::srgb(0.25, 0.22, 0.18)),
            selected: Some(Color::srgb(0.22, 0.25, 0.30)),
        }
    }

    /// Compute color for combination of states
    pub fn resolve(
        &self,
        interaction: Interaction,
        is_active: bool,
        is_selected: bool,
        is_disabled: bool,
    ) -> Color {
        if is_disabled {
            return self.disabled;
        }

        // Base color priority: selected > active > normal
        let base = if is_selected {
            self.selected.unwrap_or(self.normal)
        } else if is_active {
            self.active.unwrap_or(self.normal)
        } else {
            self.normal
        };

        match interaction {
            Interaction::Pressed => self.pressed,
            Interaction::Hovered => {
                // Lighten the base color on hover
                if is_selected || is_active {
                    Self::lighten(base, 0.08)
                } else {
                    self.hovered
                }
            }
            Interaction::None => base,
        }
    }

    fn lighten(color: Color, amount: f32) -> Color {
        let Srgba {
            red,
            green,
            blue,
            alpha,
        } = color.to_srgba();
        Color::srgba(
            (red + amount).min(1.0),
            (green + amount).min(1.0),
            (blue + amount).min(1.0),
            alpha,
        )
    }
}

// ============================================================
// BorderStyle
// ============================================================

/// Border colors for different interaction states.
///
/// Opt-in component: add to elements that need border feedback.
/// Works independently of `InteractiveVisual` (background feedback).
#[derive(Component, Clone, Debug)]
pub struct BorderStyle {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
    pub active: Option<Color>,
    pub selected: Option<Color>,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            normal: Color::srgb(0.35, 0.35, 0.40),
            hovered: Color::srgb(0.45, 0.45, 0.50),
            pressed: Color::srgb(0.40, 0.40, 0.45),
            disabled: Color::srgb(0.25, 0.25, 0.30),
            active: None,
            selected: None,
        }
    }
}

impl BorderStyle {
    pub fn new(normal: Color, hovered: Color, pressed: Color, disabled: Color) -> Self {
        Self {
            normal,
            hovered,
            pressed,
            disabled,
            active: None,
            selected: None,
        }
    }

    pub fn with_active(mut self, active: Color) -> Self {
        self.active = Some(active);
        self
    }

    pub fn with_selected(mut self, selected: Color) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Border style for inventory/equipment slots
    pub fn slot() -> Self {
        Self {
            normal: Color::srgb(0.35, 0.35, 0.40),
            hovered: Color::srgb(0.50, 0.50, 0.55),
            pressed: Color::srgb(0.45, 0.45, 0.50),
            disabled: Color::srgb(0.25, 0.25, 0.30),
            active: Some(Color::srgb(0.60, 0.50, 0.30)),
            selected: Some(Color::srgb(0.85, 0.75, 0.35)), // Golden border
        }
    }

    /// Border style for list items
    pub fn list_item() -> Self {
        Self {
            normal: Color::srgba(0.0, 0.0, 0.0, 0.0), // Invisible by default
            hovered: Color::srgb(0.40, 0.40, 0.45),
            pressed: Color::srgb(0.35, 0.35, 0.40),
            disabled: Color::srgba(0.0, 0.0, 0.0, 0.0),
            active: None,
            selected: Some(Color::srgb(0.50, 0.65, 0.85)), // Blue border
        }
    }

    /// Compute border color for combination of states
    pub fn resolve(
        &self,
        interaction: Interaction,
        is_active: bool,
        is_selected: bool,
        is_disabled: bool,
    ) -> Color {
        if is_disabled {
            return self.disabled;
        }

        // Selected takes priority over active for border
        if is_selected {
            if let Some(selected_color) = self.selected {
                return match interaction {
                    Interaction::Hovered => Self::lighten(selected_color, 0.1),
                    _ => selected_color,
                };
            }
        }

        if is_active {
            if let Some(active_color) = self.active {
                return match interaction {
                    Interaction::Hovered => Self::lighten(active_color, 0.08),
                    _ => active_color,
                };
            }
        }

        match interaction {
            Interaction::Pressed => self.pressed,
            Interaction::Hovered => self.hovered,
            Interaction::None => self.normal,
        }
    }

    fn lighten(color: Color, amount: f32) -> Color {
        let Srgba {
            red,
            green,
            blue,
            alpha,
        } = color.to_srgba();
        Color::srgba(
            (red + amount).min(1.0),
            (green + amount).min(1.0),
            (blue + amount).min(1.0),
            alpha,
        )
    }
}

// ============================================================
// Systems
// ============================================================

/// System: update BackgroundColor based on interaction state.
///
/// Only processes entities with `InteractiveVisual` marker.
#[allow(clippy::type_complexity)]
pub(crate) fn update_interactive_visuals(
    global_style: Res<ButtonStyle>,
    mut query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            Has<Disabled>,
            Has<Active>,
            Has<Selected>,
            Option<&VisualStyle>,
        ),
        With<InteractiveVisual>,
    >,
    interaction_changed: Query<Entity, (Changed<Interaction>, With<InteractiveVisual>)>,
    active_added: Query<Entity, (Added<Active>, With<InteractiveVisual>)>,
    selected_added: Query<Entity, (Added<Selected>, With<InteractiveVisual>)>,
    mut removed_active: RemovedComponents<Active>,
    mut removed_selected: RemovedComponents<Selected>,
) {
    // Collect all entities that need update
    let mut to_update: Vec<Entity> = Vec::new();

    for entity in &interaction_changed {
        to_update.push(entity);
    }

    for entity in &active_added {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in &selected_added {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in removed_active.read() {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in removed_selected.read() {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    // Update collected entities
    for entity in to_update {
        if let Ok((_, interaction, mut bg, is_disabled, is_active, is_selected, local_style)) =
            query.get_mut(entity)
        {
            *bg = BackgroundColor(compute_bg_color(
                &global_style,
                local_style,
                *interaction,
                is_active,
                is_selected,
                is_disabled,
            ));
        }
    }
}

fn compute_bg_color(
    global_style: &ButtonStyle,
    local_style: Option<&VisualStyle>,
    interaction: Interaction,
    is_active: bool,
    is_selected: bool,
    is_disabled: bool,
) -> Color {
    if let Some(style) = local_style {
        style.resolve(interaction, is_active, is_selected, is_disabled)
    } else {
        // Global style doesn't support selected, treat as normal
        if is_disabled {
            global_style.disabled
        } else {
            match interaction {
                Interaction::Pressed => global_style.pressed,
                Interaction::Hovered => global_style.hovered,
                Interaction::None => global_style.normal,
            }
        }
    }
}

/// System: update BorderColor based on interaction state.
///
/// Only processes entities with `BorderStyle` component.
#[allow(clippy::type_complexity)]
pub(crate) fn update_border_visuals(
    mut query: Query<(
        Entity,
        &Interaction,
        &mut BorderColor,
        &BorderStyle,
        Has<Disabled>,
        Has<Active>,
        Has<Selected>,
    )>,
    interaction_changed: Query<Entity, (Changed<Interaction>, With<BorderStyle>)>,
    active_added: Query<Entity, (Added<Active>, With<BorderStyle>)>,
    selected_added: Query<Entity, (Added<Selected>, With<BorderStyle>)>,
    mut removed_active: RemovedComponents<Active>,
    mut removed_selected: RemovedComponents<Selected>,
) {
    // Collect all entities that need update
    let mut to_update: Vec<Entity> = Vec::new();

    for entity in &interaction_changed {
        to_update.push(entity);
    }

    for entity in &active_added {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in &selected_added {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in removed_active.read() {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    for entity in removed_selected.read() {
        if !to_update.contains(&entity) {
            to_update.push(entity);
        }
    }

    // Update collected entities
    for entity in to_update {
        if let Ok((_, interaction, mut border, style, is_disabled, is_active, is_selected)) =
            query.get_mut(entity)
        {
            *border = BorderColor(style.resolve(*interaction, is_active, is_selected, is_disabled));
        }
    }
}
