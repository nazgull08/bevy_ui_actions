use bevy::prelude::*;

/// Trait for defining an action triggered by UI interaction.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_ui_actions::UiAction;
///
/// #[derive(Resource, Default)]
/// struct Counter(i32);
///
/// struct IncrementAction { amount: i32 }
///
/// impl UiAction for IncrementAction {
///     fn execute(&self, world: &mut World) {
///         world.resource_mut::<Counter>().0 += self.amount;
///     }
/// }
/// ```
///
/// # Execution Model
///
/// Actions are executed via `Commands::queue()`, which means:
/// - Execution happens at the end of the current frame
/// - Exclusive `World` access is guaranteed
/// - Safe for the Bevy scheduler
pub trait UiAction: Send + Sync + 'static {
    /// Execute the action with full `World` access.
    fn execute(&self, world: &mut World);
}
