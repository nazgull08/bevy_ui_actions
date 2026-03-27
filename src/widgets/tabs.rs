use bevy::prelude::*;

use super::Active;

/// Tab group container — stores the index of the active tab.
#[derive(Component, Default)]
pub struct TabGroup {
    pub active: usize,
}

impl TabGroup {
    pub fn new(active: usize) -> Self {
        Self { active }
    }
}

/// A tab button.
#[derive(Component)]
pub struct Tab {
    pub index: usize,
}

impl Tab {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

/// Tab content panel — visible when `tab.index == group.active`.
#[derive(Component)]
pub struct TabContent {
    pub index: usize,
}

impl TabContent {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

/// System: clicking a [`Tab`] updates `TabGroup.active`.
/// Walks up the hierarchy to find the parent [`TabGroup`].
pub(crate) fn handle_tab_clicks(
    tab_query: Query<(Entity, &Interaction, &Tab, &ChildOf), Changed<Interaction>>,
    parent_query: Query<&ChildOf>,
    mut group_query: Query<&mut TabGroup>,
) {
    for (_, interaction, tab, parent) in &tab_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Walk up the hierarchy to find TabGroup
        let mut current = parent.parent();

        for _ in 0..10 {
            if let Ok(mut group) = group_query.get_mut(current) {
                if group.active != tab.index {
                    group.active = tab.index;
                }
                break;
            }

            if let Ok(next_parent) = parent_query.get(current) {
                current = next_parent.parent();
            } else {
                break;
            }
        }
    }
}

/// System: syncs [`TabContent`] visibility.
/// Uses `Display::None` so hidden content does not occupy layout space.
pub(crate) fn sync_tab_content_visibility(
    group_query: Query<(Entity, &TabGroup), Changed<TabGroup>>,
    children_query: Query<&Children>,
    mut content_query: Query<(&TabContent, &mut Node)>,
) {
    for (group_entity, group) in &group_query {
        let mut to_visit = vec![group_entity];

        while let Some(entity) = to_visit.pop() {
            if let Ok((content, mut node)) = content_query.get_mut(entity) {
                node.display = if content.index == group.active {
                    Display::Flex
                } else {
                    Display::None
                };
            }

            if let Ok(children) = children_query.get(entity) {
                to_visit.extend(children.iter());
            }
        }
    }
}

/// System: inserts/removes [`Active`] marker on the active tab.
/// Searches [`Tab`] components recursively down the hierarchy.
pub(crate) fn sync_active_tab_marker(
    group_query: Query<(Entity, &TabGroup), Changed<TabGroup>>,
    children_query: Query<&Children>,
    tab_query: Query<&Tab>,
    mut commands: Commands,
) {
    for (group_entity, group) in &group_query {
        let mut to_visit = vec![group_entity];

        while let Some(entity) = to_visit.pop() {
            if let Ok(tab) = tab_query.get(entity) {
                if tab.index == group.active {
                    commands.entity(entity).insert(Active);
                } else {
                    commands.entity(entity).remove::<Active>();
                }
            }

            if let Ok(children) = children_query.get(entity) {
                to_visit.extend(children.iter());
            }
        }
    }
}
