use bevy::prelude::*;

/// Progress bar container.
#[derive(Component)]
pub struct ProgressBar {
    /// Value in the 0.0..=1.0 range.
    pub value: f32,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    pub fn empty() -> Self {
        Self { value: 0.0 }
    }

    pub fn full() -> Self {
        Self { value: 1.0 }
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

/// Marker for the fill element inside a [`ProgressBar`].
#[derive(Component)]
pub struct ProgressBarFill;

/// Configuration for spawning a progress bar.
#[derive(Clone, Debug)]
pub struct ProgressBarConfig {
    pub width: Val,
    pub height: Val,
    pub background: Color,
    pub fill_color: Color,
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for ProgressBarConfig {
    fn default() -> Self {
        Self {
            width: Val::Px(100.0),
            height: Val::Px(16.0),
            background: Color::srgb(0.15, 0.15, 0.18),
            fill_color: Color::srgb(0.3, 0.7, 0.3),
            border_width: 1.0,
            border_color: Color::srgb(0.3, 0.3, 0.35),
        }
    }
}

impl ProgressBarConfig {
    /// Health bar preset.
    pub fn health() -> Self {
        Self {
            fill_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        }
    }

    /// Mana bar preset.
    pub fn mana() -> Self {
        Self {
            fill_color: Color::srgb(0.2, 0.3, 0.8),
            ..default()
        }
    }

    /// Stamina bar preset.
    pub fn stamina() -> Self {
        Self {
            fill_color: Color::srgb(0.2, 0.7, 0.3),
            ..default()
        }
    }

    /// Attribute bar preset.
    pub fn attribute() -> Self {
        Self {
            height: Val::Px(12.0),
            fill_color: Color::srgb(0.6, 0.6, 0.3),
            ..default()
        }
    }
}

/// Extension trait for spawning a progress bar.
pub trait SpawnProgressBarExt {
    fn spawn_progress_bar(&mut self, config: ProgressBarConfig, initial_value: f32) -> Entity;
}

impl SpawnProgressBarExt for ChildSpawnerCommands<'_> {
    fn spawn_progress_bar(&mut self, config: ProgressBarConfig, initial_value: f32) -> Entity {
        self.spawn((
            Node {
                width: config.width,
                height: config.height,
                border: UiRect::all(Val::Px(config.border_width)),
                ..default()
            },
            BackgroundColor(config.background),
            BorderColor(config.border_color),
            ProgressBar::new(initial_value),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(initial_value * 100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(config.fill_color),
                ProgressBarFill,
            ));
        })
        .id()
    }
}

/// System: updates fill width when [`ProgressBar::value`] changes.
pub(crate) fn update_progress_bars(
    bar_query: Query<(&ProgressBar, &Children), Changed<ProgressBar>>,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
) {
    for (bar, children) in &bar_query {
        for child in children.iter() {
            if let Ok(mut node) = fill_query.get_mut(child) {
                node.width = Val::Percent(bar.value * 100.0);
            }
        }
    }
}
