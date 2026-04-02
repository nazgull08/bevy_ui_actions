use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{TextRole, UiAction, UiInputScope, UiTextExt};
use crate::widgets::hypertext::{HyperLinkClicked, HyperTextConfig, SpawnHyperTextExt};
use crate::widgets::panel::{PanelConfig, SpawnPanelExt};
use crate::widgets::scroll_view::ScrollView;

// ============================================================
// Config
// ============================================================

/// Position of the dialogue box on screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialoguePosition {
    #[default]
    Bottom,
    Top,
    Center,
}

/// Visual and behavioral configuration for the dialogue box.
#[derive(Clone, Debug)]
pub struct DialogueConfig {
    /// Screen position.
    pub position: DialoguePosition,
    /// Panel visual config.
    pub panel: PanelConfig,
    /// Hypertext config for dialogue text.
    pub hypertext: HyperTextConfig,
    /// Text role for speaker name.
    pub speaker_role: TextRole,
    /// Text role for topic headers (when appending topic responses).
    pub topic_header_role: TextRole,
    /// Whether ESC dismisses the dialogue.
    pub close_on_esc: bool,
    /// Height of the dialogue panel.
    pub height: Val,
    /// Width of the dialogue panel.
    pub width: Val,
    /// Show topic list panel on the right (Morrowind-style).
    /// Only effective when `TopicRegistry` resource exists.
    pub show_topic_panel: bool,
    /// Width of the topic panel (right side).
    pub topic_panel_width: Val,
    /// Text role for topic list items.
    pub topic_list_role: TextRole,
}

impl Default for DialogueConfig {
    fn default() -> Self {
        Self {
            position: DialoguePosition::Bottom,
            panel: PanelConfig {
                background: Color::srgba(0.08, 0.08, 0.10, 0.95),
                border_color: Color::srgb(0.35, 0.35, 0.40),
                border_width: 2.0,
                padding: 20.0,
                gap: 10.0,
                direction: FlexDirection::Column,
                ..PanelConfig::dark()
            },
            hypertext: HyperTextConfig::default(),
            speaker_role: TextRole::Heading,
            topic_header_role: TextRole::Heading,
            close_on_esc: true,
            height: Val::Px(300.0),
            width: Val::Percent(80.0),
            show_topic_panel: true,
            topic_panel_width: Val::Px(200.0),
            topic_list_role: TextRole::Body,
        }
    }
}

// ============================================================
// Request / Queue
// ============================================================

/// A request to show a dialogue.
pub struct DialogueRequest {
    /// Speaker name (displayed above text). `None` = no speaker line.
    pub speaker: Option<String>,
    /// Dialogue text with `[Display|key]` hyperlink markup.
    pub text: String,
    /// Config override. If `None`, uses `DialogueStyle` resource.
    pub config: Option<DialogueConfig>,
    /// Action to run when the dialogue is dismissed.
    pub on_close: Option<Arc<dyn UiAction>>,
}

impl DialogueRequest {
    /// Create a simple dialogue request.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            speaker: None,
            text: text.into(),
            config: None,
            on_close: None,
        }
    }

    /// Set speaker name (chainable).
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Set config (chainable).
    pub fn with_config(mut self, config: DialogueConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set on_close action (chainable).
    pub fn with_on_close(mut self, action: impl UiAction) -> Self {
        self.on_close = Some(Arc::new(action));
        self
    }
}

/// Resource queue for dialogue requests.
#[derive(Resource, Default)]
pub struct DialogueQueue {
    pub(crate) pending: Vec<DialogueRequest>,
}

impl DialogueQueue {
    /// Queue a dialogue to be shown next frame.
    pub fn show(&mut self, request: DialogueRequest) {
        self.pending.push(request);
    }
}

// ============================================================
// Style resource
// ============================================================

/// Default dialogue config, used when `DialogueRequest.config` is `None`.
#[derive(Resource, Clone, Debug, Default)]
pub struct DialogueStyle(pub DialogueConfig);

// ============================================================
// Topic Registry
// ============================================================

/// A single entry in the topic registry.
#[derive(Clone, Debug)]
pub struct TopicEntry {
    /// Display title (shown as header when appended to dialogue).
    pub title: String,
    /// Response text (supports `[Display|key]` hyperlink markup).
    pub text: String,
    /// Optional category for grouping (e.g. "locations", "characters", "quests").
    pub category: Option<String>,
    /// Whether this topic has been discovered (viewed) by the player.
    pub discovered: bool,
}

impl TopicEntry {
    /// Create a new topic entry.
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            text: text.into(),
            category: None,
            discovered: false,
        }
    }

    /// Set category (chainable).
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Optional resource for automatic topic resolution in dialogues.
///
/// When present, [`handle_dialogue_topic`] will look up topics from
/// `HyperLinkClicked` events and automatically append responses
/// to the active dialogue. Without this resource, game code must
/// handle `HyperLinkClicked` events manually.
#[derive(Resource, Default)]
pub struct TopicRegistry {
    entries: HashMap<String, TopicEntry>,
}

impl TopicRegistry {
    /// Insert a topic entry.
    pub fn insert(&mut self, key: impl Into<String>, entry: TopicEntry) {
        self.entries.insert(key.into(), entry);
    }

    /// Get a topic entry by key.
    pub fn get(&self, key: &str) -> Option<&TopicEntry> {
        self.entries.get(key)
    }

    /// Get a mutable topic entry by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut TopicEntry> {
        self.entries.get_mut(key)
    }

    /// Mark a topic as discovered.
    pub fn discover(&mut self, key: &str) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.discovered = true;
        }
    }

    /// Check if a topic has been discovered.
    pub fn is_discovered(&self, key: &str) -> bool {
        self.entries.get(key).is_some_and(|e| e.discovered)
    }

    /// Get all entries in a given category.
    pub fn by_category(&self, category: &str) -> Vec<(&str, &TopicEntry)> {
        self.entries
            .iter()
            .filter(|(_, e)| e.category.as_deref() == Some(category))
            .map(|(k, e)| (k.as_str(), e))
            .collect()
    }

    /// Get all discovered entries.
    pub fn discovered(&self) -> Vec<(&str, &TopicEntry)> {
        self.entries
            .iter()
            .filter(|(_, e)| e.discovered)
            .map(|(k, e)| (k.as_str(), e))
            .collect()
    }

    /// Get all entries.
    pub fn all(&self) -> Vec<(&str, &TopicEntry)> {
        self.entries.iter().map(|(k, e)| (k.as_str(), e)).collect()
    }
}

/// Event fired when a topic is discovered (first time viewed) via the dialogue system.
#[derive(Event, Debug, Clone)]
pub struct TopicDiscovered {
    /// The topic key that was discovered.
    pub topic: String,
}

// ============================================================
// Components
// ============================================================

/// Marker on the dialogue box root entity (full-screen overlay).
#[derive(Component)]
pub struct DialogueBox {
    pub on_close: Option<Arc<dyn UiAction>>,
    pub close_on_esc: bool,
    /// Stored config for appending topic responses.
    pub config: DialogueConfig,
}

/// Marker on the inner column container where text blocks are appended.
#[derive(Component)]
pub struct DialogueContent;

/// Marker on the ScrollView entity inside the dialogue box.
#[derive(Component)]
pub struct DialogueScroll;

/// Marker on the right-side topic list panel (Morrowind-style).
/// Only spawned when `TopicRegistry` resource exists.
#[derive(Component)]
pub struct DialogueTopicPanel;

/// Marker on individual topic buttons in the topic panel.
#[derive(Component)]
pub struct DialogueTopicButton {
    /// The topic key this button represents.
    pub topic: String,
}

/// Event: dismiss the current dialogue.
#[derive(Event)]
pub struct DismissDialogueEvent;

/// Action that dismisses the current dialogue.
pub struct DismissDialogue;

impl UiAction for DismissDialogue {
    fn execute(&self, world: &mut World) {
        world.send_event(DismissDialogueEvent);
    }
}

// ============================================================
// Systems
// ============================================================

/// Processes queued dialogue requests.
pub(crate) fn process_dialogue_queue(
    mut queue: ResMut<DialogueQueue>,
    mut commands: Commands,
    style: Res<DialogueStyle>,
    existing: Query<Entity, With<DialogueBox>>,
    registry: Option<Res<TopicRegistry>>,
) {
    let Some(request) = queue.pending.pop() else {
        return;
    };
    // One dialogue at a time
    queue.pending.clear();

    // Despawn existing
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let config = request.config.unwrap_or_else(|| style.0.clone());

    // Collect discovered topics for topic panel
    let discovered_topics: Vec<(String, String)> = if config.show_topic_panel {
        registry
            .as_ref()
            .map(|reg| {
                let mut topics: Vec<_> = reg
                    .discovered()
                    .iter()
                    .map(|(key, entry)| (key.to_string(), entry.title.clone()))
                    .collect();
                topics.sort_by(|a, b| a.1.cmp(&b.1));
                topics
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let dialogue_entity = spawn_dialogue(
        &mut commands,
        &config,
        request.speaker,
        &request.text,
        request.on_close,
        &discovered_topics,
    );
    commands.insert_resource(UiInputScope { root: dialogue_entity });
}

/// Handles ESC to dismiss dialogue.
pub(crate) fn handle_dialogue_dismiss_input(
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<&DialogueBox>,
    mut events: EventWriter<DismissDialogueEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for dialogue in &query {
            if dialogue.close_on_esc {
                events.write(DismissDialogueEvent);
                return;
            }
        }
    }
}

/// Processes DismissDialogueEvent: fires on_close action and despawns.
pub(crate) fn handle_dialogue_dismiss_event(
    mut events: EventReader<DismissDialogueEvent>,
    query: Query<(Entity, &DialogueBox)>,
    mut commands: Commands,
    scope: Option<Res<UiInputScope>>,
) {
    for _event in events.read() {
        for (entity, dialogue) in &query {
            if let Some(ref action) = dialogue.on_close {
                let action = action.clone();
                commands.queue(move |world: &mut World| {
                    action.execute(world);
                });
            }

            commands.entity(entity).despawn();

            if scope.is_some() {
                commands.remove_resource::<UiInputScope>();
            }
        }
    }
}

/// Listens for HyperLinkClicked events and appends topic responses
/// from TopicRegistry (if available) into the dialogue scroll area.
///
/// Without a `TopicRegistry` resource, this system does nothing —
/// game code should handle `HyperLinkClicked` events directly
/// and call [`append_dialogue_text`] with the response text.
pub(crate) fn handle_dialogue_topic(
    mut link_events: EventReader<HyperLinkClicked>,
    dialogue_query: Query<&DialogueBox>,
    content_query: Query<Entity, With<DialogueContent>>,
    mut scroll_query: Query<&mut ScrollPosition, With<DialogueScroll>>,
    mut commands: Commands,
    registry: Option<ResMut<TopicRegistry>>,
    mut discovered_events: EventWriter<TopicDiscovered>,
) {
    let Some(mut registry) = registry else {
        // No TopicRegistry — drain events, game code handles them.
        for _event in link_events.read() {}
        return;
    };

    for event in link_events.read() {
        let Some(entry) = registry.get(&event.topic).cloned() else {
            continue;
        };

        let Ok(content_entity) = content_query.single() else {
            continue;
        };
        let Ok(mut scroll_pos) = scroll_query.single_mut() else {
            continue;
        };
        let Ok(dialogue) = dialogue_query.single() else {
            continue;
        };

        // Fire discovered event on first view
        if !entry.discovered {
            discovered_events.write(TopicDiscovered {
                topic: event.topic.clone(),
            });
        }
        registry.discover(&event.topic);

        append_dialogue_text(
            &mut commands,
            content_entity,
            &mut scroll_pos,
            &dialogue.config,
            &entry.title,
            &entry.text,
        );
    }
}

/// Handles clicks on topic buttons in the right panel.
/// Converts button clicks into `HyperLinkClicked` events.
pub(crate) fn handle_topic_panel_clicks(
    query: Query<(&Interaction, &DialogueTopicButton), Changed<Interaction>>,
    dialogue_query: Query<Entity, With<DialogueBox>>,
    mut events: EventWriter<HyperLinkClicked>,
) {
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let source = dialogue_query.single().unwrap_or(Entity::PLACEHOLDER);
        events.write(HyperLinkClicked {
            topic: button.topic.clone(),
            source,
        });
    }
}

/// Hover highlight for topic panel buttons — changes text color on hover.
#[allow(clippy::type_complexity)]
pub(crate) fn topic_button_hover(
    query: Query<(&Interaction, &Children), (Changed<Interaction>, With<DialogueTopicButton>)>,
    dialogue_query: Query<&DialogueBox>,
    mut text_colors: Query<&mut TextColor>,
) {
    if query.is_empty() {
        return;
    }
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    let link_color = dialogue.config.hypertext.link_color;
    let hover_color = dialogue.config.hypertext.link_hover_color;

    for (interaction, children) in &query {
        let color = match interaction {
            Interaction::Hovered | Interaction::Pressed => hover_color,
            Interaction::None => link_color,
        };
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(child) {
                tc.0 = color;
            }
        }
    }
}

/// Updates the topic panel when new topics are discovered.
/// Rebuilds the button list sorted alphabetically by title.
pub(crate) fn update_topic_panel(
    mut discovered_events: EventReader<TopicDiscovered>,
    panel_query: Query<(Entity, &Children), With<DialogueTopicPanel>>,
    existing_buttons: Query<Entity, With<DialogueTopicButton>>,
    dialogue_query: Query<&DialogueBox>,
    registry: Option<Res<TopicRegistry>>,
    mut commands: Commands,
) {
    let events: Vec<_> = discovered_events.read().collect();
    if events.is_empty() {
        return;
    }

    let Ok((panel_entity, panel_children)) = panel_query.single() else {
        return;
    };
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    let Some(registry) = registry else {
        return;
    };

    // Despawn existing buttons (keep header + divider)
    for child in panel_children.iter() {
        if existing_buttons.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }

    // Collect all discovered topics, sorted by title
    let mut topics: Vec<(String, String)> = registry
        .discovered()
        .iter()
        .map(|(key, entry)| (key.to_string(), entry.title.clone()))
        .collect();
    topics.sort_by(|a, b| a.1.cmp(&b.1));

    let topic_list_role = dialogue.config.topic_list_role;
    let link_color = dialogue.config.hypertext.link_color;

    commands.entity(panel_entity).with_children(|col| {
        for (key, title) in &topics {
            col.spawn((
                DialogueTopicButton {
                    topic: key.clone(),
                },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                    ..default()
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(title.clone()),
                    TextFont {
                        font_size: topic_list_role.size(),
                        ..default()
                    },
                    TextColor(link_color),
                    crate::core::UiThemedText,
                ));
            });
        }
    });
}

/// Run condition: true when any DialogueBox entities exist.
pub fn has_dialogue(query: Query<(), With<DialogueBox>>) -> bool {
    !query.is_empty()
}

// ============================================================
// Spawn helpers
// ============================================================

fn spawn_dialogue(
    commands: &mut Commands,
    config: &DialogueConfig,
    speaker: Option<String>,
    text: &str,
    on_close: Option<Arc<dyn UiAction>>,
    discovered_topics: &[(String, String)], // (key, title)
) -> Entity {
    let justify = match config.position {
        DialoguePosition::Bottom => JustifyContent::FlexEnd,
        DialoguePosition::Top => JustifyContent::FlexStart,
        DialoguePosition::Center => JustifyContent::Center,
    };

    let panel_config = config.panel.clone();
    let hypertext_config = config.hypertext.clone();
    let speaker_role = config.speaker_role;
    let close_on_esc = config.close_on_esc;
    let height = config.height;
    let width = config.width;
    let show_topic_panel = config.show_topic_panel;
    let topic_panel_width = config.topic_panel_width;
    let topic_list_role = config.topic_list_role;
    let link_color = config.hypertext.link_color;

    let speaker_text = speaker.clone();
    let text_owned = text.to_string();
    let config_stored = config.clone();
    let topics_owned: Vec<(String, String)> = discovered_topics.to_vec();

    // Full-screen overlay
    commands
        .spawn((
            DialogueBox {
                on_close,
                close_on_esc,
                config: config_stored,
            },
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: justify,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(800),
        ))
        .with_children(|overlay| {
            // Panel with overflow hidden so scroll stays within bounds
            overlay
                .spawn_panel(PanelConfig {
                    width,
                    height,
                    ..panel_config
                })
                .with_children(|panel| {
                    // Speaker name
                    if let Some(ref name) = speaker_text {
                        panel.ui_text(speaker_role, name);
                    }

                    // Main row: [scroll area] | [topic panel]
                    let main_row = panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            min_height: Val::Px(0.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .id();

                    // --- Left side: scroll wrapper with scrollbar ---
                    let scroll_wrapper = panel
                        .commands()
                        .spawn(Node {
                            flex_grow: 1.0,
                            min_height: Val::Px(0.0),
                            flex_direction: FlexDirection::Row,
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .id();

                    let hypertext_cfg = hypertext_config.clone();
                    let text = text_owned.clone();
                    let scrollbar_width = 14.0;
                    let track_color = Color::srgba(0.15, 0.15, 0.18, 0.5);
                    let thumb_color = Color::srgba(0.5, 0.5, 0.55, 0.6);

                    // ScrollView
                    let scroll_entity = panel
                        .commands()
                        .spawn((
                            DialogueScroll,
                            ScrollView {
                                direction: crate::widgets::scroll_view::ScrollDirection::Vertical,
                                scroll_speed: 40.0,
                            },
                            ScrollPosition::default(),
                            Interaction::None,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                overflow: Overflow {
                                    x: OverflowAxis::Clip,
                                    y: OverflowAxis::Scroll,
                                },
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ))
                        .id();

                    // DialogueContent column inside scroll
                    let content_entity = panel
                        .commands()
                        .spawn((
                            DialogueContent,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                        ))
                        .id();

                    // Initial hypertext
                    panel.commands().entity(content_entity).with_children(|col| {
                        col.spawn_hypertext(&hypertext_cfg, &text);
                    });

                    // Scrollbar track
                    let track_entity = panel
                        .commands()
                        .spawn((
                            crate::widgets::scroll_view::ScrollbarTrack {
                                scroll_view: scroll_entity,
                            },
                            Node {
                                width: Val::Px(scrollbar_width),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(track_color),
                            Interaction::None,
                        ))
                        .id();

                    // Scrollbar thumb
                    let thumb_entity = panel
                        .commands()
                        .spawn((
                            crate::widgets::scroll_view::ScrollbarThumb {
                                scroll_view: scroll_entity,
                            },
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                position_type: PositionType::Absolute,
                                top: Val::Px(0.0),
                                left: Val::Px(0.0),
                                ..default()
                            },
                            BackgroundColor(thumb_color),
                            Interaction::None,
                            BorderRadius::all(Val::Px(scrollbar_width / 2.0)),
                        ))
                        .id();

                    // Assemble scroll: wrapper → [scroll → [content], track → [thumb]]
                    panel.commands().entity(scroll_entity).add_child(content_entity);
                    panel.commands().entity(track_entity).add_child(thumb_entity);
                    panel
                        .commands()
                        .entity(scroll_wrapper)
                        .add_child(scroll_entity)
                        .add_child(track_entity);

                    // Add scroll wrapper to main row
                    panel.commands().entity(main_row).add_child(scroll_wrapper);

                    // --- Right side: topic panel ---
                    if show_topic_panel {
                        let divider = panel
                            .commands()
                            .spawn((
                                Node {
                                    width: Val::Px(1.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.5)),
                            ))
                            .id();

                        let topic_panel = panel
                            .commands()
                            .spawn((
                                DialogueTopicPanel,
                                Node {
                                    width: topic_panel_width,
                                    min_width: topic_panel_width,
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    overflow: Overflow {
                                        x: OverflowAxis::Clip,
                                        y: OverflowAxis::Scroll,
                                    },
                                    row_gap: Val::Px(2.0),
                                    padding: UiRect::left(Val::Px(8.0)),
                                    ..default()
                                },
                                ScrollView {
                                    direction: crate::widgets::scroll_view::ScrollDirection::Vertical,
                                    scroll_speed: 30.0,
                                },
                                ScrollPosition::default(),
                                Interaction::None,
                            ))
                            .id();

                        // Spawn topic buttons
                        panel.commands().entity(topic_panel).with_children(|col| {
                            // Header
                            col.ui_text(topic_list_role, "Topics");
                            col.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(1.0),
                                margin: UiRect::vertical(Val::Px(4.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.3)));

                            for (key, title) in &topics_owned {
                                col.spawn((
                                    DialogueTopicButton {
                                        topic: key.clone(),
                                    },
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(title.clone()),
                                        TextFont {
                                            font_size: topic_list_role.size(),
                                            ..default()
                                        },
                                        TextColor(link_color),
                                        crate::core::UiThemedText,
                                    ));
                                });
                            }
                        });

                        panel
                            .commands()
                            .entity(main_row)
                            .add_child(divider)
                            .add_child(topic_panel);
                    }
                });
        })
        .id()
}

// ============================================================
// Public API for appending text
// ============================================================

/// Append a new text block to the dialogue's scroll area.
///
/// `content_entity` is the entity with [`DialogueContent`] marker (inner column).
/// `scroll_pos` is the [`ScrollPosition`] on the [`DialogueScroll`] entity (ScrollView).
///
/// Use this from game code when handling `HyperLinkClicked` events
/// to add topic responses to the conversation.
///
/// ```ignore
/// fn handle_topic(
///     mut events: EventReader<HyperLinkClicked>,
///     mut commands: Commands,
///     content: Query<Entity, With<DialogueContent>>,
///     mut scroll: Query<&mut ScrollPosition, With<DialogueScroll>>,
///     dialogue: Query<&DialogueBox>,
/// ) {
///     for event in events.read() {
///         let response = my_topic_lookup(&event.topic);
///         let content_entity = content.single().unwrap();
///         let mut scroll_pos = scroll.single_mut().unwrap();
///         append_dialogue_text(&mut commands, content_entity, &mut scroll_pos, &dialogue.config, &event.topic, &response);
///     }
/// }
/// ```
pub fn append_dialogue_text(
    commands: &mut Commands,
    content_entity: Entity,
    scroll_pos: &mut ScrollPosition,
    config: &DialogueConfig,
    header: &str,
    text: &str,
) {
    commands.entity(content_entity).with_children(|content| {
        content
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Px(12.0)),
                ..default()
            })
            .with_children(|block| {
                // Topic header
                block.ui_text(config.topic_header_role, header);
                // Response text with hyperlinks
                block.spawn_hypertext(&config.hypertext, text);
            });
    });

    // Auto-scroll to bottom
    scroll_pos.offset_y = f32::MAX;
}
