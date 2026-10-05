use bevy::prelude::*;

use super::Active;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_groups_keep_visibility_and_active_markers_independent() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (sync_tab_content_visibility, sync_active_tab_marker),
        );
        let outer = app.world_mut().spawn(TabGroup::new(0)).id();
        let inner = app
            .world_mut()
            .spawn((
                TabGroup::new(1),
                TabContent::new(0),
                Node::default(),
                ChildOf(outer),
            ))
            .id();
        let outer_tab = app.world_mut().spawn((Tab::new(0), ChildOf(outer))).id();
        let inner_tab = app.world_mut().spawn((Tab::new(1), ChildOf(inner))).id();
        let inner_content = app
            .world_mut()
            .spawn((TabContent::new(1), Node::default(), ChildOf(inner)))
            .id();
        app.update();
        assert!(app.world().get::<Active>(outer_tab).is_some());
        assert!(app.world().get::<Active>(inner_tab).is_some());
        assert_eq!(
            app.world().get::<Node>(inner).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(inner_content).unwrap().display,
            Display::Flex
        );

        app.world_mut().get_mut::<TabGroup>(outer).unwrap().active = 2;
        app.update();
        assert_eq!(
            app.world().get::<Node>(inner).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(inner_content).unwrap().display,
            Display::Flex
        );
        assert!(app.world().get::<Active>(outer_tab).is_none());
        assert!(app.world().get::<Active>(inner_tab).is_some());

        app.world_mut().get_mut::<TabGroup>(inner).unwrap().active = 0;
        app.update();
        assert_eq!(
            app.world().get::<Node>(inner).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(inner_content).unwrap().display,
            Display::None
        );
        assert!(app.world().get::<Active>(inner_tab).is_none());
    }

    #[test]
    fn click_at_depth_more_than_ten_targets_nearest_group() {
        let mut app = App::new();
        app.add_systems(Update, handle_tab_clicks);
        let outer = app.world_mut().spawn(TabGroup::new(0)).id();
        let inner = app
            .world_mut()
            .spawn((TabGroup::new(1), ChildOf(outer)))
            .id();
        let mut parent = inner;
        for _ in 0..12 {
            parent = app.world_mut().spawn(ChildOf(parent)).id();
        }
        app.world_mut()
            .spawn((Tab::new(2), Interaction::Pressed, ChildOf(parent)));
        app.update();
        assert_eq!(app.world().get::<TabGroup>(inner).unwrap().active, 2);
        assert_eq!(app.world().get::<TabGroup>(outer).unwrap().active, 0);
    }
}

/// Tab group container — stores the index of the active tab.
/// Nested groups own their descendants independently of the outer group.
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

        loop {
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
    all_groups: Query<(), With<TabGroup>>,
    mut content_query: Query<(&TabContent, &mut Node)>,
) {
    for (group_entity, group) in &group_query {
        // The group's own TabContent/Tab (if present) belongs to its parent.
        let mut to_visit = children_query
            .get(group_entity)
            .map(|children| children.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        while let Some(entity) = to_visit.pop() {
            if let Ok((content, mut node)) = content_query.get_mut(entity) {
                node.display = if content.index == group.active {
                    Display::Flex
                } else {
                    Display::None
                };
            }

            // A nested group's container can itself be an outer TabContent.
            // Its descendants belong to the nested group, even if that group
            // did not change this frame.
            if all_groups.contains(entity) {
                continue;
            }

            if let Ok(children) = children_query.get(entity) {
                to_visit.extend(children.iter());
            }
        }
    }
}

/// System: inserts/removes [`Active`] marker on the active tab.
/// Searches [`Tab`] components recursively within the nearest group.
pub(crate) fn sync_active_tab_marker(
    group_query: Query<(Entity, &TabGroup), Changed<TabGroup>>,
    children_query: Query<&Children>,
    all_groups: Query<(), With<TabGroup>>,
    tab_query: Query<&Tab>,
    mut commands: Commands,
) {
    for (group_entity, group) in &group_query {
        let mut to_visit = children_query
            .get(group_entity)
            .map(|children| children.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        while let Some(entity) = to_visit.pop() {
            if let Ok(tab) = tab_query.get(entity) {
                if tab.index == group.active {
                    commands.entity(entity).insert(Active);
                } else {
                    commands.entity(entity).remove::<Active>();
                }
            }

            if all_groups.contains(entity) {
                continue;
            }

            if let Ok(children) = children_query.get(entity) {
                to_visit.extend(children.iter());
            }
        }
    }
}
