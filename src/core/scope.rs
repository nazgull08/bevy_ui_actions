use bevy::prelude::*;

/// When present, only entities that are descendants of `root` (or `root` itself)
/// receive interactions (clicks, hover, drag, scroll, visual feedback).
///
/// Insert this resource to restrict all UI input to a subtree (e.g., modal overlay).
/// Remove it to restore normal input to all elements.
#[derive(Resource)]
pub struct UiInputScope {
    pub root: Entity,
}

/// Check if `entity` is `scope.root` or a descendant of it.
///
/// Walks up the `ChildOf` hierarchy. Returns `true` if `entity` is within scope.
/// O(depth) — typically 5-15 for UI hierarchies.
pub fn is_in_scope(entity: Entity, scope: &UiInputScope, parents: &Query<&ChildOf>) -> bool {
    if entity == scope.root {
        return true;
    }
    let mut current = entity;
    while let Ok(child_of) = parents.get(current) {
        let parent = child_of.parent();
        if parent == scope.root {
            return true;
        }
        current = parent;
    }
    false
}
