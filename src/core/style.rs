use bevy::prelude::*;

/// Colors for button interaction states.
#[derive(Clone, Debug, Resource)]
pub struct ButtonStyle {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal: Color::srgb(0.2, 0.2, 0.2),
            hovered: Color::srgb(0.3, 0.3, 0.3),
            pressed: Color::srgb(0.1, 0.1, 0.1),
            disabled: Color::srgb(0.1, 0.1, 0.1),
        }
    }
}
