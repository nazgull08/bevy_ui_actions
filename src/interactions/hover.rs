use crate::core::UiAction;
use crate::widgets::Disabled;
use bevy::prelude::*;
use std::sync::Arc;

/// Action triggered when the cursor enters the element.
#[derive(Component)]
pub struct OnHover {
    action: Arc<dyn UiAction>,
}

impl OnHover {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }

    pub(crate) fn execute(&self, commands: &mut Commands) {
        let action = self.action.clone();
        commands.queue(move |world: &mut World| {
            action.execute(world);
        });
    }
}

/// Action triggered when the cursor leaves the element.
#[derive(Component)]
pub struct OnHoverExit {
    action: Arc<dyn UiAction>,
}

impl OnHoverExit {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }

    pub(crate) fn execute(&self, commands: &mut Commands) {
        let action = self.action.clone();
        commands.queue(move |world: &mut World| {
            action.execute(world);
        });
    }
}

/// Action triggered on press (before release).
#[derive(Component)]
pub struct OnPress {
    action: Arc<dyn UiAction>,
}

impl OnPress {
    pub fn new(action: impl UiAction) -> Self {
        Self {
            action: Arc::new(action),
        }
    }

    pub(crate) fn execute(&self, commands: &mut Commands) {
        let action = self.action.clone();
        commands.queue(move |world: &mut World| {
            action.execute(world);
        });
    }
}

/// Tracks previous `Interaction` state for detecting transitions.
#[derive(Component, Default)]
pub struct PreviousInteraction(pub Interaction);

// ============ Systems ============

/// System: fires [`OnHover`] when entering `Interaction::Hovered`.
#[allow(clippy::type_complexity)]
pub(crate) fn handle_hover_actions(
    query: Query<(&Interaction, &OnHover), (Changed<Interaction>, Without<Disabled>)>,
    mut commands: Commands,
) {
    for (interaction, on_hover) in &query {
        if *interaction == Interaction::Hovered {
            on_hover.execute(&mut commands);
        }
    }
}

/// System: fires [`OnHoverExit`] when leaving `Interaction::Hovered`.
pub(crate) fn handle_hover_exit_actions(
    mut query: Query<(&Interaction, &mut PreviousInteraction, &OnHoverExit), Without<Disabled>>,
    mut commands: Commands,
) {
    for (interaction, mut prev, on_hover_exit) in &mut query {
        if prev.0 == Interaction::Hovered && *interaction != Interaction::Hovered {
            on_hover_exit.execute(&mut commands);
        }
        prev.0 = *interaction;
    }
}

/// System: fires [`OnPress`] when entering `Interaction::Pressed`.
#[allow(clippy::type_complexity)]
pub(crate) fn handle_press_actions(
    query: Query<(&Interaction, &OnPress), (Changed<Interaction>, Without<Disabled>)>,
    mut commands: Commands,
) {
    for (interaction, on_press) in &query {
        if *interaction == Interaction::Pressed {
            on_press.execute(&mut commands);
        }
    }
}
