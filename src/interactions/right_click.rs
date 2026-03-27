use crate::core::UiAction;
use crate::widgets::Disabled;
use bevy::prelude::*;
use std::sync::Arc;

/// Action triggered on right mouse button click.
#[derive(Component)]
pub struct OnRightClick {
    action: Arc<dyn UiAction>,
}

impl OnRightClick {
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

/// System: fires [`OnRightClick`] action when right-clicking a hovered element.
pub(crate) fn handle_right_clicks(
    query: Query<(&Interaction, &OnRightClick), Without<Disabled>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Right) {
        for (interaction, on_right_click) in &query {
            if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
                on_right_click.execute(&mut commands);
            }
        }
    }
}
