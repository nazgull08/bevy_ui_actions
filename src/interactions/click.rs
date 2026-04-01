use crate::core::{is_in_scope, UiAction, UiInputScope};
use crate::widgets::Disabled;
use bevy::prelude::*;
use std::sync::Arc;

/// Action triggered on left mouse button click.
#[derive(Component)]
pub struct OnClick {
    action: Arc<dyn UiAction>,
}

impl OnClick {
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

/// System: fires [`OnClick`] action when `Interaction::Pressed`.
#[allow(clippy::type_complexity)]
pub(crate) fn handle_clicks(
    query: Query<(Entity, &Interaction, &OnClick), (Changed<Interaction>, Without<Disabled>)>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (entity, interaction, on_click) in &query {
        if *interaction == Interaction::Pressed {
            if let Some(ref scope) = scope {
                if !is_in_scope(entity, scope, &parents) {
                    continue;
                }
            }
            on_click.execute(&mut commands);
        }
    }
}
