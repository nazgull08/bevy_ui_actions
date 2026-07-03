use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{TextRole, UiAction, UiInputScope, UiTextExt, ZLayer};
use crate::widgets::hypertext::{
    append_topic_block, HyperLinkClicked, HyperTextConfig, SpawnHyperTextExt, TopicContainer,
};
use crate::widgets::panel::{PanelConfig, SpawnPanelExt};
use crate::widgets::scroll_view::{ScrollView, StickToBottom};

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

/// How a conversation node is drawn as the player navigates topics/choices.
///
/// The dialogue *data* (nodes, topics, choices, effects) is identical regardless
/// of presentation — this only changes rendering, mirroring how mature systems
/// (Ink, Yarn Spinner, Articy) split dialogue data from its view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialoguePresentation {
    /// Morrowind / Disco Elysium: a scroll log accumulates topic responses; the
    /// choice row updates in place at the bottom. This is the implemented mode.
    #[default]
    Append,
    /// BioWare / Fallout node-view: each node replaces the screen. **TODO** —
    /// reserved variant; currently falls back to [`Append`].
    ///
    /// [`Append`]: DialoguePresentation::Append
    Replace,
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
    /// Text color for the currently-selected (active) topic in the topic panel.
    /// The last-clicked topic's button rests in this color instead of `link_color`
    /// (Morrowind-style selected-topic highlight). Hover still overrides it.
    pub topic_active_color: Color,
    /// Text role for choice buttons (the answer list at the bottom).
    pub choice_role: TextRole,
    /// Vertical gap between choice buttons (pixels).
    pub choice_gap: f32,
    /// Text color of an enabled choice.
    pub choice_text_color: Color,
    /// Text color for a disabled (non-selectable) choice.
    pub choice_disabled_color: Color,
    /// Choice button background — resting / hover / pressed / disabled.
    /// The visible background swap on hover/press is what makes a click read as
    /// "registered" (text-color alone is too subtle).
    pub choice_bg: Color,
    pub choice_bg_hover: Color,
    pub choice_bg_pressed: Color,
    pub choice_bg_disabled: Color,
    /// Choice button border — resting / hover.
    pub choice_border: Color,
    pub choice_border_hover: Color,
    /// Corner radius of choice buttons (pixels).
    pub choice_border_radius: f32,
    /// Allow selecting a choice by pressing its number key (`1`–`9`).
    pub choice_hotkeys: bool,
    /// Prefix choice labels with their number (`"1. Ask…"`) so the hotkeys are
    /// discoverable. Independent of [`choice_hotkeys`](Self::choice_hotkeys).
    pub choice_number_labels: bool,
    /// How nodes are drawn as topics/choices are navigated. `Replace` is TODO —
    /// it falls back to `Append`.
    pub presentation: DialoguePresentation,
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
            height: Val::Px(380.0),
            width: Val::Percent(80.0),
            show_topic_panel: true,
            topic_panel_width: Val::Px(200.0),
            topic_list_role: TextRole::Body,
            topic_active_color: Color::srgb(0.9, 0.8, 0.45),
            choice_role: TextRole::Body,
            choice_gap: 6.0,
            choice_text_color: Color::srgb(0.88, 0.92, 1.0),
            choice_disabled_color: Color::srgb(0.45, 0.45, 0.5),
            choice_bg: Color::srgb(0.16, 0.18, 0.24),
            choice_bg_hover: Color::srgb(0.28, 0.33, 0.44),
            choice_bg_pressed: Color::srgb(0.42, 0.50, 0.66),
            choice_bg_disabled: Color::srgb(0.11, 0.11, 0.13),
            choice_border: Color::srgb(0.34, 0.37, 0.45),
            choice_border_hover: Color::srgb(0.55, 0.66, 0.9),
            choice_border_radius: 4.0,
            choice_hotkeys: true,
            choice_number_labels: true,
            presentation: DialoguePresentation::Append,
        }
    }
}

// ============================================================
// Request / Queue
// ============================================================

/// One selectable answer in a dialogue (the "choices where needed" layer).
///
/// The library renders these as a button column and emits [`DialogueChoiceSelected`]
/// with the chosen `key` — it never interprets the key itself. Game code maps the
/// key to whatever it means (open trade, set a flag, go to another node, end).
#[derive(Clone, Debug)]
pub struct DialogueChoice {
    /// Button label shown to the player.
    pub label: String,
    /// Stable, game-defined identifier returned in `DialogueChoiceSelected`.
    pub key: String,
    /// When `false`, the choice is greyed out and cannot be selected.
    pub enabled: bool,
}

impl DialogueChoice {
    /// Create an enabled choice.
    pub fn new(label: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            enabled: true,
        }
    }

    /// Mark the choice disabled (greyed, non-selectable) — chainable.
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// A request to show a dialogue.
pub struct DialogueRequest {
    /// Speaker name (displayed above text). `None` = no speaker line.
    pub speaker: Option<String>,
    /// Dialogue text with `[Display|key]` hyperlink markup.
    pub text: String,
    /// Answer choices shown at the bottom. Empty = pure topic dialogue.
    pub choices: Vec<DialogueChoice>,
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
            choices: Vec::new(),
            config: None,
            on_close: None,
        }
    }

    /// Set speaker name (chainable).
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Set answer choices (chainable).
    pub fn with_choices(mut self, choices: Vec<DialogueChoice>) -> Self {
        self.choices = choices;
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

/// The currently-selected (active) topic key, if any.
///
/// Set to the last topic opened via a hyperlink or topic-panel button; reset when
/// a new dialogue box is spawned. Drives the active-topic highlight in the topic
/// panel (see [`update_topic_button_colors`]).
#[derive(Resource, Default)]
pub struct ActiveTopic(pub Option<String>);

/// When `true`, topic navigation is suspended: topic-panel clicks and text-link
/// auto-append do nothing, and topic buttons render greyed. Consumers set this
/// while a decision is pending (choices shown) so the player must resolve the
/// choice before browsing topics again. Reset to `false` on a new dialogue box.
#[derive(Resource, Default)]
pub struct DialogueTopicsLocked(pub bool);

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
/// When present, [`handle_topic_container`](crate::widgets::hypertext::handle_topic_container)
/// will look up topics from `HyperLinkClicked` events and automatically
/// append responses to any [`TopicContainer`]. Without this resource,
/// game code should handle `HyperLinkClicked` events manually.
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

/// Marker on the choice button column at the bottom of the dialogue box.
#[derive(Component)]
pub struct DialogueChoicesRow;

/// Marker on an individual choice button.
#[derive(Component)]
pub struct DialogueChoiceButton {
    /// Game-defined key returned when this choice is selected.
    pub key: String,
    /// Whether the choice is selectable (disabled buttons emit nothing).
    pub enabled: bool,
    /// 1-based position in the row — the number key that selects this choice.
    pub number: usize,
}

/// Event: the player selected a dialogue choice. Game code maps `key` → effect.
#[derive(Event, Debug, Clone)]
pub struct DialogueChoiceSelected {
    /// The `key` of the selected [`DialogueChoice`].
    pub key: String,
}

/// Event: replace the current answer choices *in place*, without rebuilding the box
/// (so the scroll log is preserved). This is the topic-nodes primitive — open a
/// topic → show its choices; resolve a choice → swap or clear them.
///
/// An empty `choices` list clears the row (back to pure topic mode). Only the most
/// recent event per frame is applied.
#[derive(Event, Debug, Clone, Default)]
pub struct SetDialogueChoices {
    /// The new set of answers. Empty = clear.
    pub choices: Vec<DialogueChoice>,
}

impl SetDialogueChoices {
    /// Show the given choices.
    pub fn new(choices: Vec<DialogueChoice>) -> Self {
        Self { choices }
    }

    /// Clear all choices (pure topic mode).
    pub fn clear() -> Self {
        Self { choices: Vec::new() }
    }
}

/// Event: append a text block to the dialogue scroll log and pin the view to the
/// new bottom. This is the append primitive — used for choice outcomes, forced
/// reveals, or any narration added mid-conversation.
///
/// `text` supports `[Display|key]` hyperlink markup (so an appended block can
/// surface new topic links). `header` renders an optional bold title above it.
#[derive(Event, Debug, Clone)]
pub struct AppendDialogueText {
    /// Optional bold header line above the text.
    pub header: Option<String>,
    /// Body text with `[Display|key]` hyperlink markup.
    pub text: String,
}

impl AppendDialogueText {
    /// Append plain narration (no header).
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            header: None,
            text: text.into(),
        }
    }

    /// Add a header line above the text (chainable).
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }
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
    mut active_topic: ResMut<ActiveTopic>,
    mut topics_locked: ResMut<DialogueTopicsLocked>,
) {
    let Some(request) = queue.pending.pop() else {
        return;
    };
    // One dialogue at a time
    queue.pending.clear();
    // Fresh box → no topic selected, topics browsable.
    active_topic.0 = None;
    topics_locked.0 = false;

    // Despawn existing
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let mut config = request.config.unwrap_or_else(|| style.0.clone());

    // `Replace` presentation is not implemented yet — silently render as `Append`
    // so callers still get a working box. The crate is logging-free by design, and
    // `Replace` is a documented TODO variant. See DialoguePresentation docs.
    if config.presentation == DialoguePresentation::Replace {
        config.presentation = DialoguePresentation::Append;
    }

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
        &request.choices,
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

// NOTE: Topic resolution (HyperLinkClicked → TopicRegistry → append)
// is now handled by the generic `handle_topic_container` system in hypertext.rs.
// DialogueContent gets a `TopicContainer` component at spawn time for backward compat.

/// Handles clicks on topic buttons in the right panel.
/// Converts button clicks into `HyperLinkClicked` events.
///
/// The event `source` must be the [`DialogueContent`] entity (which carries the
/// [`TopicContainer`]), because `handle_topic_container` searches *upward* from
/// `source` for the container. Emitting the root `DialogueBox` instead — as this
/// did — leaves the search with nowhere to climb, so panel clicks silently no-op
/// while text-link clicks (whose source is inside the content) work.
pub(crate) fn handle_topic_panel_clicks(
    query: Query<(&Interaction, &DialogueTopicButton), Changed<Interaction>>,
    content_query: Query<Entity, With<DialogueContent>>,
    locked: Res<DialogueTopicsLocked>,
    mut events: EventWriter<HyperLinkClicked>,
) {
    // Topics suspended while a decision is pending.
    if locked.0 {
        return;
    }
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let source = content_query.single().unwrap_or(Entity::PLACEHOLDER);
        events.write(HyperLinkClicked {
            topic: button.topic.clone(),
            source,
        });
    }
}

/// Records the last-opened topic as [`ActiveTopic`]. Both text-link clicks and
/// topic-panel button clicks funnel through [`HyperLinkClicked`], so tracking it
/// here covers both. A non-topic link key simply matches no panel button (harmless).
pub(crate) fn track_active_topic(
    mut events: EventReader<HyperLinkClicked>,
    mut active: ResMut<ActiveTopic>,
) {
    for event in events.read() {
        active.0 = Some(event.topic.clone());
    }
}

/// Owns the resting/hover color of every topic-panel button.
///
/// Color = hover ? hover : (this is the active topic ? active_color : link_color).
/// A single system prevents the hover and active-highlight logic from fighting over
/// `TextColor`. Runs every frame (topic lists are short) but only writes on change.
pub(crate) fn update_topic_button_colors(
    query: Query<(&Interaction, &DialogueTopicButton, &Children)>,
    dialogue_query: Query<&DialogueBox>,
    active: Res<ActiveTopic>,
    locked: Res<DialogueTopicsLocked>,
    mut text_colors: Query<&mut TextColor>,
) {
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    let link_color = dialogue.config.hypertext.link_color;
    let hover_color = dialogue.config.hypertext.link_hover_color;
    let active_color = dialogue.config.topic_active_color;
    let locked_color = dialogue.config.choice_disabled_color;

    for (interaction, button, children) in &query {
        let is_active = active.0.as_deref() == Some(button.topic.as_str());
        // While locked, topics read as unavailable — greyed, no hover/active tint.
        let color = if locked.0 {
            locked_color
        } else {
            match interaction {
                Interaction::Hovered | Interaction::Pressed => hover_color,
                Interaction::None if is_active => active_color,
                Interaction::None => link_color,
            }
        };
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(child) {
                if tc.0 != color {
                    tc.0 = color;
                }
            }
        }
    }
}

/// Handles clicks on choice buttons → emits [`DialogueChoiceSelected`].
/// Disabled choices emit nothing. Game code maps the key to an effect.
pub(crate) fn handle_choice_clicks(
    query: Query<(&Interaction, &DialogueChoiceButton), Changed<Interaction>>,
    mut events: EventWriter<DialogueChoiceSelected>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed && button.enabled {
            events.write(DialogueChoiceSelected {
                key: button.key.clone(),
            });
        }
    }
}

/// Selects a choice by number key (`1`–`9`) — same effect as clicking it. The key
/// maps to the choice's 1-based row position; disabled positions do nothing.
/// Gated by `config.choice_hotkeys`.
pub(crate) fn handle_choice_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    dialogue_query: Query<&DialogueBox>,
    buttons: Query<&DialogueChoiceButton>,
    mut events: EventWriter<DialogueChoiceSelected>,
) {
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    if !dialogue.config.choice_hotkeys {
        return;
    }
    const DIGITS: [(KeyCode, usize); 9] = [
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ];
    for (code, number) in DIGITS {
        if !keys.just_pressed(code) {
            continue;
        }
        for button in &buttons {
            if button.number == number && button.enabled {
                events.write(DialogueChoiceSelected {
                    key: button.key.clone(),
                });
            }
        }
    }
}

/// Applies [`SetDialogueChoices`]: clears the current choice buttons and repopulates
/// the always-present [`DialogueChoicesRow`] anchor in place. Only the latest event
/// this frame is applied (the choice set is a state, not a stream).
pub(crate) fn apply_set_choices(
    mut events: EventReader<SetDialogueChoices>,
    row_query: Query<(Entity, Option<&Children>), With<DialogueChoicesRow>>,
    existing_buttons: Query<Entity, With<DialogueChoiceButton>>,
    dialogue_query: Query<&DialogueBox>,
    mut commands: Commands,
) {
    let Some(request) = events.read().last() else {
        return;
    };
    let Ok((row_entity, children)) = row_query.single() else {
        return;
    };
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    let config = dialogue.config.clone();

    // Despawn the existing choice buttons (deferred; runs before the spawns below).
    if let Some(children) = children {
        for child in children.iter() {
            if existing_buttons.get(child).is_ok() {
                commands.entity(child).despawn();
            }
        }
    }

    // Repopulate with the requested set (empty → row left blank).
    let choices = request.choices.clone();
    commands.entity(row_entity).with_children(|col| {
        for (i, choice) in choices.iter().enumerate() {
            spawn_choice_button(col, &config, choice, i + 1);
        }
    });
}

/// Applies [`AppendDialogueText`]: appends each requested block to the dialogue's
/// content column and pins the scroll to the (new) bottom via a [`StickToBottom`]
/// latch — the same correct path the automatic topic append uses. A one-shot
/// `offset_y = MAX` would clamp to the *old* height; the latch re-pins until layout
/// settles.
pub(crate) fn apply_append_text(
    mut events: EventReader<AppendDialogueText>,
    content_query: Query<(Entity, &TopicContainer), With<DialogueContent>>,
    parent_query: Query<&ChildOf>,
    scroll_query: Query<(), With<ScrollView>>,
    mut commands: Commands,
) {
    let Ok((content_entity, container)) = content_query.single() else {
        return;
    };
    let container = container.clone();

    let mut appended = false;
    for event in events.read() {
        append_topic_block(
            &mut commands,
            content_entity,
            &container,
            event.header.as_deref(),
            &event.text,
        );
        appended = true;
    }
    if !appended {
        return;
    }

    // Walk up from the content column to the enclosing ScrollView and pin it.
    let mut walk = content_entity;
    loop {
        if scroll_query.get(walk).is_ok() {
            commands.entity(walk).insert(StickToBottom::default());
            break;
        }
        if let Ok(child_of) = parent_query.get(walk) {
            walk = child_of.parent();
        } else {
            break;
        }
    }
}

/// Drives the full visual state of choice buttons on interaction — background,
/// border and text color for resting / hover / pressed / disabled.
///
/// The visible **background** swap (and a brighter pressed shade) is the point: it
/// makes a click read as registered, which subtle text-only recoloring did not.
/// Disabled buttons stay in their greyed style regardless of hover.
pub(crate) fn update_choice_button_visuals(
    query: Query<(Entity, &Interaction, &DialogueChoiceButton, &Children), Changed<Interaction>>,
    dialogue_query: Query<&DialogueBox>,
    mut styles: Query<(&mut BackgroundColor, &mut BorderColor)>,
    mut text_colors: Query<&mut TextColor>,
) {
    if query.is_empty() {
        return;
    }
    let Ok(dialogue) = dialogue_query.single() else {
        return;
    };
    let c = &dialogue.config;

    for (entity, interaction, button, children) in &query {
        let (bg, border, text) = if !button.enabled {
            (c.choice_bg_disabled, c.choice_border, c.choice_disabled_color)
        } else {
            match interaction {
                Interaction::Pressed => {
                    (c.choice_bg_pressed, c.choice_border_hover, c.choice_text_color)
                }
                Interaction::Hovered => {
                    (c.choice_bg_hover, c.choice_border_hover, c.choice_text_color)
                }
                Interaction::None => (c.choice_bg, c.choice_border, c.choice_text_color),
            }
        };
        if let Ok((mut background, mut border_color)) = styles.get_mut(entity) {
            if background.0 != bg {
                background.0 = bg;
            }
            if border_color.0 != border {
                border_color.0 = border;
            }
        }
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(child) {
                if tc.0 != text {
                    tc.0 = text;
                }
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

/// Spawn one styled choice button as a child of `col`. Shared by the initial box
/// build and [`apply_set_choices`] so a choice looks identical however it appears.
fn spawn_choice_button(
    col: &mut ChildSpawnerCommands,
    config: &DialogueConfig,
    choice: &DialogueChoice,
    number: usize,
) {
    let (text_color, bg_color) = if choice.enabled {
        (config.choice_text_color, config.choice_bg)
    } else {
        (config.choice_disabled_color, config.choice_bg_disabled)
    };
    // Answers track the dialogue body's font size (fallback: the choice role's
    // default) so they never look smaller than the text they answer.
    let font_size = config
        .hypertext
        .font_size
        .unwrap_or_else(|| config.choice_role.size());
    let label = if config.choice_number_labels {
        format!("{number}. {}", choice.label)
    } else {
        choice.label.clone()
    };
    col.spawn((
        DialogueChoiceButton {
            key: choice.key.clone(),
            enabled: choice.enabled,
            number,
        },
        Button,
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        BorderColor(config.choice_border),
        BorderRadius::all(Val::Px(config.choice_border_radius)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(text_color),
            crate::core::UiThemedText,
        ));
    });
}

fn spawn_dialogue(
    commands: &mut Commands,
    config: &DialogueConfig,
    speaker: Option<String>,
    text: &str,
    choices: &[DialogueChoice],
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
    let choice_gap = config.choice_gap;
    let choices_owned: Vec<DialogueChoice> = choices.to_vec();

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
            GlobalZIndex(ZLayer::Dialogue.z()),
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

                    // Main row: [ left column: text + choices ] | [topic panel]
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

                    // Left column: dialogue text (scroll) on top, the choices
                    // container beneath it. Keeps choices under the text — the topic
                    // panel stays full-height on the right, never overlapped.
                    let left_col = panel
                        .commands()
                        .spawn(Node {
                            flex_grow: 1.0,
                            min_height: Val::Px(0.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
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
                            TopicContainer {
                                hypertext_config: hypertext_config.clone(),
                                header_role: config.topic_header_role,
                            },
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

                    // Scroll text fills the top of the left column; left column is
                    // the first (leftmost) child of the main row.
                    panel.commands().entity(left_col).add_child(scroll_wrapper);
                    panel.commands().entity(main_row).add_child(left_col);

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

                    // Choices column — the "answers where needed" layer, pinned to
                    // the bottom of the left column (directly under the dialogue
                    // text). ALWAYS spawned (even empty) so it is a stable anchor
                    // `SetDialogueChoices` can populate later. Empty → zero height.
                    let choices_row = panel
                        .commands()
                        .spawn((
                            DialogueChoicesRow,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.0,
                                row_gap: Val::Px(choice_gap),
                                margin: UiRect::top(Val::Px(10.0)),
                                ..default()
                            },
                        ))
                        .with_children(|col| {
                            for (i, choice) in choices_owned.iter().enumerate() {
                                spawn_choice_button(col, config, choice, i + 1);
                            }
                        })
                        .id();
                    panel.commands().entity(left_col).add_child(choices_row);
                });
        })
        .id()
}

