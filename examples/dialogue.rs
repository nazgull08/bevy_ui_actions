//! Dialogue box example.
//!
//! Demonstrates:
//! - Dialogue box with speaker name and hypertext
//! - TopicRegistry: automatic topic navigation via hyperlinks
//! - Visited link coloring (discovered topics shown in different color)
//! - Active-topic highlight: the last-opened topic is highlighted in the panel
//! - Topic-nodes flow: opening the "Sealed Vaults" topic presents answer choices
//!   (SetDialogueChoices); picking one clears them (DialogueChoiceSelected)
//! - ESC to dismiss, auto-scroll on append
//!
//! Run: `cargo run --example dialogue -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                open_dialogue,
                log_discoveries,
                present_vault_choices,
                resolve_choice,
            ),
        )
        .run();
}

#[derive(Component)]
struct HintText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Populate TopicRegistry with lore entries
    let mut registry = TopicRegistry::default();

    registry.insert("library", TopicEntry::new(
        "Ancient Library",
        "The Ancient Library was founded three centuries ago by the [First Archivists|archivists]. Its halls stretch deep beneath the mountain, holding scrolls from every corner of the known world. The deepest vaults are sealed by the [Iron Council|council].",
    ).with_category("locations"));

    registry.insert("crystal_spire", TopicEntry::new(
        "Crystal Spire",
        "The Crystal Spire rises at the center of the old city. It was built during the [Golden Age|golden_age] as a beacon of knowledge. Some say it still resonates with the voices of the ancients. The [Iron Council|council] forbids anyone from entering its upper chambers.",
    ).with_category("locations"));

    registry.insert("war", TopicEntry::new(
        "Forgotten War",
        "The Forgotten War lasted forty years and nearly destroyed civilization. It began when the [Southern Kingdoms|kingdoms] refused the authority of the [Iron Council|council]. The war ended only when the [Crystal Spire|crystal_spire] unleashed a great pulse of light, silencing all combatants.",
    ).with_category("events"));

    registry.insert("council", TopicEntry::new(
        "Iron Council",
        "The Iron Council is a body of seven elders who govern the allied cities. They were established after the [Forgotten War|war] to prevent such conflict from ever recurring. Their seat of power lies within the [Ancient Library|library] itself.",
    ).with_category("factions"));

    registry.insert("archivists", TopicEntry::new(
        "First Archivists",
        "The First Archivists were scholars who survived the [Forgotten War|war]. They gathered every surviving text and founded the [Ancient Library|library] to preserve knowledge for future generations.",
    ).with_category("factions"));

    registry.insert("golden_age", TopicEntry::new(
        "Golden Age",
        "The Golden Age preceded the [Forgotten War|war] by two centuries. It was a time of great prosperity and magical discovery. The [Crystal Spire|crystal_spire] was the crowning achievement of that era.",
    ).with_category("events"));

    registry.insert("kingdoms", TopicEntry::new(
        "Southern Kingdoms",
        "The Southern Kingdoms were a loose confederation of city-states. Their refusal to submit to the [Iron Council|council] sparked the [Forgotten War|war]. Today, most have been absorbed into the allied cities.",
    ).with_category("factions"));

    // A "decision" topic: opening it presents choices (see `present_vault_choices`).
    registry.insert("vaults", TopicEntry::new(
        "The Sealed Vaults",
        "The deepest vaults hold the most dangerous knowledge, sealed by the [Iron Council|council]. Only the Archivist may grant passage. Will you request it?",
    ).with_category("locations"));

    commands.insert_resource(registry);

    // UI
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        })
        .with_children(|root| {
            root.ui_text(TextRole::Heading, "Dialogue Box Demo");

            root.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.25, 0.35)),
                OnClick::new(OpenDialogue),
                InteractiveVisual,
            ))
            .with_children(|btn| {
                btn.ui_text(TextRole::Button, "Talk to Librarian");
            });

            root.ui_text(
                TextRole::Caption,
                "Click topics to append responses — the log auto-scrolls to the newest. ESC dismisses.",
            )
            .insert(HintText);
        });
}

// ============ Actions ============

struct OpenDialogue;

impl UiAction for OpenDialogue {
    fn execute(&self, world: &mut World) {
        let mut queue = world.resource_mut::<DialogueQueue>();
        queue.show(
            DialogueRequest::new(
                "Welcome to the [Ancient Library|library], traveler. Here you will find records of the [Crystal Spire|crystal_spire], the [Forgotten War|war], and the edicts of the [Iron Council|council]. You may also request access to the [Sealed Vaults|vaults]. What would you like to know about?",
            )
            .with_speaker("Archivist Maren")
            // Starts in pure topic mode — no choices. Opening the "vaults" topic
            // presents a decision (see `present_vault_choices`).
            .with_config(DialogueConfig {
                hypertext: HyperTextConfig {
                    font_size: Some(18.0),
                    ..default()
                },
                height: Val::Px(460.0),
                ..default()
            }),
        );
    }
}

fn open_dialogue() {
    // Placeholder — the button action handles this via UiAction
}

/// Log when topics are discovered (game code would use this for quests, codex, etc.)
fn log_discoveries(mut events: EventReader<TopicDiscovered>) {
    for event in events.read() {
        info!("Topic discovered: '{}'", event.topic);
    }
}

/// Topic-node flow: opening the "vaults" topic presents a decision. Both topic-panel
/// clicks and in-text `[Sealed Vaults|vaults]` links funnel through `HyperLinkClicked`,
/// so this catches either. The library also appends the topic's lore (auto-append).
fn present_vault_choices(
    mut links: EventReader<HyperLinkClicked>,
    mut set_choices: EventWriter<SetDialogueChoices>,
    mut topics_locked: ResMut<DialogueTopicsLocked>,
) {
    for link in links.read() {
        if link.topic == "vaults" {
            set_choices.write(SetDialogueChoices::new(vec![
                DialogueChoice::new("Ask the Archivist for passage", "vault_enter"),
                DialogueChoice::new("Never mind", "vault_cancel"),
            ]));
            // A decision is pending — lock topics until it's resolved.
            topics_locked.0 = true;
        }
    }
}

/// Resolve a choice: log the outcome (a real game would append text / set a flag /
/// open trade), clear the choices and unlock topics so browsing resumes.
fn resolve_choice(
    mut events: EventReader<DialogueChoiceSelected>,
    mut set_choices: EventWriter<SetDialogueChoices>,
    mut append: EventWriter<AppendDialogueText>,
    mut topics_locked: ResMut<DialogueTopicsLocked>,
) {
    for event in events.read() {
        let result = match event.key.as_str() {
            "vault_enter" => {
                "The Archivist studies you for a long moment. \"The vaults remain sealed, \
                 traveler. Not for such as you.\""
            }
            "vault_cancel" => "You nod and step back from the request.",
            _ => continue,
        };
        // Visible outcome in the log (auto-scrolls to it).
        append.write(AppendDialogueText::new(result));
        // Decision made → clear choices and resume browsing topics.
        set_choices.write(SetDialogueChoices::clear());
        topics_locked.0 = false;
    }
}
