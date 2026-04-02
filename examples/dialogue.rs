//! Dialogue box example.
//!
//! Demonstrates:
//! - Dialogue box with speaker name and hypertext
//! - TopicRegistry: automatic topic navigation via hyperlinks
//! - Visited link coloring (discovered topics shown in different color)
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
        .add_systems(Update, (open_dialogue, log_discoveries))
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

            root.ui_text(TextRole::Caption, "Press ESC to dismiss the dialogue")
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
                "Welcome to the [Ancient Library|library], traveler. Here you will find records of the [Crystal Spire|crystal_spire], the [Forgotten War|war], and the edicts of the [Iron Council|council]. What would you like to know about?",
            )
            .with_speaker("Archivist Maren")
            .with_config(DialogueConfig {
                hypertext: HyperTextConfig {
                    font_size: Some(18.0),
                    ..default()
                },
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
