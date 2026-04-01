use bevy::prelude::*;

use crate::core::UiAction;
use crate::interactions::OnClick;
use crate::widgets::scroll_view::{ScrollViewConfig, SpawnScrollViewExt};
use crate::widgets::visual::{InteractiveVisual, Selected, VisualStyle};

// ============================================================
// Types
// ============================================================

/// Selection behavior for a list view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionMode {
    /// No selection tracking (items are just clickable).
    #[default]
    None,
    /// Only one item selected at a time.
    Single,
}

/// Configuration for spawning a list view.
#[derive(Clone, Debug)]
pub struct ListViewConfig {
    pub scroll: ScrollViewConfig,
    pub selection_mode: SelectionMode,
    /// Gap between list items.
    pub item_gap: f32,
    /// Visual style for list items.
    pub item_style: VisualStyle,
}

impl Default for ListViewConfig {
    fn default() -> Self {
        Self {
            scroll: ScrollViewConfig::default(),
            selection_mode: SelectionMode::Single,
            item_gap: 2.0,
            item_style: VisualStyle {
                normal: Color::srgb(0.15, 0.15, 0.18),
                hovered: Color::srgb(0.22, 0.22, 0.25),
                pressed: Color::srgb(0.18, 0.18, 0.20),
                disabled: Color::srgb(0.1, 0.1, 0.12),
                active: None,
                selected: Some(Color::srgb(0.20, 0.25, 0.35)),
            },
        }
    }
}

/// Marker on the list view root entity.
#[derive(Component)]
pub struct ListView {
    pub selection_mode: SelectionMode,
}

/// Marker on a list item.
#[derive(Component)]
pub struct ListItem {
    pub index: usize,
    /// The ListView entity this item belongs to.
    pub list: Entity,
}

/// Event sent when a list item is selected.
#[derive(Event)]
pub struct ListItemSelected {
    /// The selected ListItem entity.
    pub entity: Entity,
    /// The item index.
    pub index: usize,
    /// The ListView entity.
    pub list: Entity,
}

// ============================================================
// Selection Action
// ============================================================

struct SelectListItemAction {
    item_entity: Entity,
    list_entity: Entity,
    index: usize,
    mode: SelectionMode,
}

impl UiAction for SelectListItemAction {
    fn execute(&self, world: &mut World) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                // Remove Selected from all items in this list
                let mut to_deselect = Vec::new();
                let mut query = world.query::<(Entity, &ListItem)>();
                for (entity, item) in query.iter(world) {
                    if item.list == self.list_entity && entity != self.item_entity {
                        to_deselect.push(entity);
                    }
                }
                for entity in to_deselect {
                    world.entity_mut(entity).remove::<Selected>();
                }

                // Toggle selection on clicked item
                let has_selected = world.entity(self.item_entity).contains::<Selected>();
                if has_selected {
                    world.entity_mut(self.item_entity).remove::<Selected>();
                } else {
                    world.entity_mut(self.item_entity).insert(Selected);
                    world.send_event(ListItemSelected {
                        entity: self.item_entity,
                        index: self.index,
                        list: self.list_entity,
                    });
                }
            }
        }
    }
}

// ============================================================
// Spawn Extension
// ============================================================

/// Extension trait for spawning list views.
pub trait SpawnListViewExt {
    /// Spawn a list view. Items are added via the callback using `ListViewItems`.
    fn spawn_list_view(
        &mut self,
        config: ListViewConfig,
        items: impl FnOnce(&mut ListViewItems),
    ) -> Entity;
}

/// Collects item definitions before spawning.
pub struct ListViewItems {
    items: Vec<ListViewItemDef>,
}

struct ListViewItemDef {
    content: Box<dyn FnOnce(&mut ChildSpawnerCommands) + Send + Sync>,
}

impl ListViewItems {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a list item. The callback defines the item's visual content.
    pub fn item(&mut self, content: impl FnOnce(&mut ChildSpawnerCommands) + Send + Sync + 'static) {
        self.items.push(ListViewItemDef {
            content: Box::new(content),
        });
    }
}

impl SpawnListViewExt for ChildSpawnerCommands<'_> {
    fn spawn_list_view(
        &mut self,
        config: ListViewConfig,
        items_fn: impl FnOnce(&mut ListViewItems),
    ) -> Entity {
        // Collect item definitions
        let mut items = ListViewItems::new();
        items_fn(&mut items);

        let selection_mode = config.selection_mode;
        let item_style = config.item_style.clone();
        let item_gap = config.item_gap;
        let scroll_config = config.scroll.clone();
        let show_scrollbar = scroll_config.show_scrollbar;

        // Helper: spawn items into a column container
        let spawn_items =
            move |parent: &mut ChildSpawnerCommands,
                  list_entity: Entity,
                  items: Vec<ListViewItemDef>| {
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(item_gap),
                        ..default()
                    })
                    .with_children(|col| {
                        for (index, item_def) in items.into_iter().enumerate() {
                            let style = item_style.clone();
                            col.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(style.normal),
                                InteractiveVisual,
                                style,
                                ListItem {
                                    index,
                                    list: list_entity,
                                },
                                // Placeholder OnClick — will be fixed up
                                OnClick::new(SelectListItemAction {
                                    item_entity: Entity::PLACEHOLDER,
                                    list_entity,
                                    index,
                                    mode: selection_mode,
                                }),
                                Interaction::None,
                            ))
                            .with_children(item_def.content);
                        }
                    });
            };

        let item_defs = items.items;

        if show_scrollbar {
            let wrapper = self.spawn_scroll_view_with(scroll_config, |scroll| {
                // Use PLACEHOLDER, fix up after
                spawn_items(scroll, Entity::PLACEHOLDER, item_defs);
            });

            self.commands()
                .entity(wrapper)
                .insert(ListView { selection_mode });

            // Fix up entity references
            self.commands().queue(move |world: &mut World| {
                fix_list_references(world, wrapper, selection_mode);
            });

            wrapper
        } else {
            let scroll_entity = {
                let mut ec = self.spawn_scroll_view(scroll_config);
                let id = ec.id();
                ec.with_children(|scroll| {
                    spawn_items(scroll, id, item_defs);
                });
                id
            };

            self.commands()
                .entity(scroll_entity)
                .insert(ListView { selection_mode });

            // Fix up OnClick actions with correct entity IDs
            self.commands().queue(move |world: &mut World| {
                fix_onclick_entities(world, scroll_entity, selection_mode);
            });

            scroll_entity
        }
    }
}

/// Fix ListItem.list references (PLACEHOLDER → real entity) and OnClick actions.
fn fix_list_references(world: &mut World, list_entity: Entity, mode: SelectionMode) {
    let mut items: Vec<(Entity, usize)> = Vec::new();
    let mut query = world.query::<(Entity, &ListItem)>();
    for (entity, item) in query.iter(world) {
        if item.list == Entity::PLACEHOLDER {
            items.push((entity, item.index));
        }
    }
    for (entity, index) in &items {
        if let Some(mut item) = world.get_mut::<ListItem>(*entity) {
            item.list = list_entity;
        }
        world.entity_mut(*entity).insert(OnClick::new(SelectListItemAction {
            item_entity: *entity,
            list_entity,
            index: *index,
            mode,
        }));
    }
}

/// Fix OnClick actions with correct entity IDs (for non-scrollbar variant).
fn fix_onclick_entities(world: &mut World, list_entity: Entity, mode: SelectionMode) {
    let mut items: Vec<(Entity, usize)> = Vec::new();
    let mut query = world.query::<(Entity, &ListItem)>();
    for (entity, item) in query.iter(world) {
        if item.list == list_entity {
            items.push((entity, item.index));
        }
    }
    for (entity, index) in &items {
        world.entity_mut(*entity).insert(OnClick::new(SelectListItemAction {
            item_entity: *entity,
            list_entity,
            index: *index,
            mode,
        }));
    }
}
