//! Hypertext example.
//!
//! Demonstrates:
//! - Text with clickable inline links
//! - Link hover highlighting
//! - HyperLinkClicked event handling
//!
//! Run: `cargo run --example hypertext -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_link_clicks)
        .run();
}

#[derive(Component)]
struct StatusText;

const BODY_SIZE: f32 = 20.0;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let config = HyperTextConfig {
        text_role: TextRole::Body,
        link_color: Color::srgb(0.4, 0.6, 0.9),
        link_hover_color: Color::srgb(0.6, 0.8, 1.0),
        visited_link_color: None,
        width: Val::Px(700.0),
        font_size: Some(BODY_SIZE),
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(25.0),
            ..default()
        })
        .with_children(|root| {
            root.ui_text(TextRole::Title, "Hypertext Demo");

            root.spawn_hypertext(
                &config,
                "Welcome, traveler. The [Ancient Library|library] holds many secrets about the [Crystal Spire|crystal_spire] and the [Forgotten War|war]. Scholars have long debated the role of the [Iron Council|council] in the fall of the [Old Empire|empire].",
            );

            root.spawn_hypertext(
                &HyperTextConfig {
                    link_color: Color::srgb(0.9, 0.6, 0.3),
                    link_hover_color: Color::srgb(1.0, 0.8, 0.5),
                    font_size: Some(BODY_SIZE),
                    ..config.clone()
                },
                "You can also use [links without pipes] where the display text equals the key. Or mix [styled links|custom] with plain text freely.",
            );

            // Status text shows clicked topic
            root.ui_text(TextRole::Body, "Click a link...")
                .insert(StatusText);
        });
}

fn handle_link_clicks(
    mut events: EventReader<HyperLinkClicked>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    for event in events.read() {
        info!(
            "Link clicked: topic='{}', source={:?}",
            event.topic, event.source
        );
        for mut text in &mut query {
            **text = format!("Clicked topic: '{}'", event.topic);
        }
    }
}
