//! ScrollView + ListView + Tabs example.
//!
//! Demonstrates:
//! - Tabs with different scroll views in each tab
//! - ListView with single selection
//! - ScrollView with scrollbar
//! - Clickable buttons inside scroll
//! - ListItemSelected event
//!
//! Run: `cargo run --example scroll_view -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_list_selection)
        .run();
}

#[derive(Component)]
struct SelectionInfoText;

// ============ Setup ============

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node::centered(20.0))
        .with_children(|root| {
            root.ui_text(TextRole::Heading, "ScrollView + Tabs Example");

            // Tab container
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(500.0),
                    ..default()
                },
                TabGroup::new(0),
            ))
            .with_children(|tab_group| {
                // Tab buttons
                tab_group
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_tab_button(row, 0, "Inventory", true);
                        spawn_tab_button(row, 1, "Quests", false);
                        spawn_tab_button(row, 2, "Actions", false);
                    });

                // Tab 0: Inventory — ListView with selection
                spawn_tab_panel(tab_group, 0, true, |panel| {
                    panel.spawn_list_view(
                        ListViewConfig {
                            scroll: ScrollViewConfig {
                                width: Val::Percent(100.0),
                                height: Val::Px(280.0),
                                show_scrollbar: true,
                                ..default()
                            },
                            selection_mode: SelectionMode::Single,
                            ..default()
                        },
                        |list| {
                            for item in ITEMS.iter() {
                                let name = item.0;
                                let desc = item.1;
                                list.item(move |row| {
                                    row.spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        col.ui_text(TextRole::Button, name);
                                        col.ui_text(TextRole::Caption, desc);
                                    });
                                });
                            }
                        },
                    );
                });

                // Tab 1: Quests — ScrollView with panels
                spawn_tab_panel(tab_group, 1, false, |panel| {
                    panel.spawn_scroll_view_with(
                        ScrollViewConfig {
                            width: Val::Percent(100.0),
                            height: Val::Px(280.0),
                            show_scrollbar: true,
                            ..default()
                        },
                        |scroll| {
                            for (title, desc) in QUESTS {
                                scroll
                                    .spawn_panel(PanelConfig {
                                        background: Color::srgb(0.14, 0.14, 0.17),
                                        border_color: Color::srgb(0.25, 0.25, 0.30),
                                        ..PanelConfig::dark()
                                    })
                                    .with_children(|quest| {
                                        quest.ui_text(TextRole::Button, *title);
                                        quest.ui_text(TextRole::Body, *desc);
                                    });
                            }
                        },
                    );
                });

                // Tab 2: Actions — clickable buttons inside scroll
                spawn_tab_panel(tab_group, 2, false, |panel| {
                    panel.spawn_scroll_view_with(
                        ScrollViewConfig {
                            width: Val::Percent(100.0),
                            height: Val::Px(280.0),
                            show_scrollbar: true,
                            ..default()
                        },
                        |scroll| {
                            for i in 0..12 {
                                spawn_action_button(scroll, i);
                            }
                        },
                    );
                });
            });

            // Selection info
            root.ui_text(TextRole::Body, "Select an item in Inventory tab")
                .insert(SelectionInfoText);

            root.ui_text(TextRole::Caption, "Tabs switch content. Each tab has a scrollable area.");
        });
}

// ============ Tab helpers ============

fn spawn_tab_button(parent: &mut ChildSpawnerCommands, index: usize, label: &str, is_active: bool) {
    let mut entity = parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 1.0,
            border: UiRect {
                left: Val::Px(1.0),
                right: Val::Px(1.0),
                top: Val::Px(1.0),
                bottom: Val::Px(0.0),
            },
            ..default()
        },
        BackgroundColor(if is_active {
            Color::srgb(0.28, 0.28, 0.32)
        } else {
            Color::srgb(0.15, 0.15, 0.18)
        }),
        BorderColor(Color::srgb(0.3, 0.3, 0.35)),
        Tab::new(index),
        VisualStyle::tab(),
        InteractiveVisual,
        Interaction::None,
    ));

    if is_active {
        entity.insert(Active);
    }

    entity.with_children(|btn| {
        btn.ui_text(TextRole::Button, label);
    });
}

fn spawn_tab_panel(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    visible: bool,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                display: if visible { Display::Flex } else { Display::None },
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor(Color::srgb(0.3, 0.3, 0.35)),
            TabContent::new(index),
        ))
        .with_children(content);
}

// ============ Action buttons ============

struct LogAction {
    index: usize,
}

impl UiAction for LogAction {
    fn execute(&self, _world: &mut World) {
        info!("Action button #{} clicked!", self.index);
    }
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, index: usize) {
    let hue = (index as f32 * 30.0) % 360.0;
    let color = Color::hsl(hue, 0.4, 0.25);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            OnClick::new(LogAction { index }),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text(TextRole::Button, format!("Action #{}", index));
        });
}

// ============ Selection handler ============

fn handle_list_selection(
    mut events: EventReader<ListItemSelected>,
    mut text_query: Query<&mut Text, With<SelectionInfoText>>,
) {
    for event in events.read() {
        if let Ok(mut text) = text_query.single_mut() {
            if event.index < ITEMS.len() {
                let (name, desc) = ITEMS[event.index];
                **text = format!("Selected: {} — {}", name, desc);
            }
        }
    }
}

// ============ Data ============

const ITEMS: &[(&str, &str)] = &[
    ("Iron Sword", "A reliable blade, standard issue"),
    ("Steel Shield", "Heavy but offers good protection"),
    ("Health Potion", "Restores 50 HP"),
    ("Mana Potion", "Restores 30 MP"),
    ("Leather Armor", "Light armor, allows quick movement"),
    ("Fire Scroll", "Casts Fireball when used"),
    ("Iron Helmet", "Basic head protection"),
    ("Stamina Potion", "Restores 40 SP"),
    ("Silver Ring", "Slightly increases magic defense"),
    ("Torch", "Illuminates dark areas"),
    ("Rope", "Useful for climbing and binding"),
    ("Lockpick Set", "Opens locked chests and doors"),
];

const QUESTS: &[(&str, &str)] = &[
    ("The Lost Sword", "Find the ancient blade hidden deep in the dungeon."),
    ("Spider Infestation", "Clear the spider nest in the lower caves."),
    ("Deliver the Message", "Bring the sealed letter to the guard captain."),
    ("Gather Herbs", "Collect 5 moonflower petals from the garden terrace."),
    ("The Missing Guard", "Investigate the disappearance of the night watch."),
    ("Ancient Mechanism", "Discover how to operate the strange device."),
    ("Repair the Bridge", "Find materials to fix the collapsed bridge."),
    ("The Sealed Door", "Find a way to open the sealed door in the cathedral."),
];
