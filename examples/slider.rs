//! Slider example — RGB color mixer + a tuning row.
//!
//! Demonstrates:
//! - Slider spawning with SliderConfig (ranges, step snap)
//! - SliderChanged events driving live state (color swatch)
//! - Programmatic Slider::set (reset button, no event echo)
//! - Composition: slider + −/+ buttons + value text (tuning-panel row)
//!
//! Run: `cargo run --example slider -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<Mix>()
        .add_systems(Startup, setup)
        .add_systems(Update, (apply_slider_changes, sync_labels, sync_swatch))
        .run();
}

/// Live state driven by the sliders.
#[derive(Resource)]
struct Mix {
    rgb: [f32; 3],
    offset: f32,
}

impl Default for Mix {
    fn default() -> Self {
        Self {
            rgb: [90.0, 140.0, 200.0],
            offset: 0.0,
        }
    }
}

/// Which channel a slider drives.
#[derive(Component, Clone, Copy)]
enum Drives {
    Channel(usize),
    Offset,
}

#[derive(Component)]
struct Swatch;

#[derive(Component)]
struct ChannelLabel(usize);

#[derive(Component)]
struct OffsetLabel;

/// −/+ nudge for the offset row: shows slider + buttons composition.
struct NudgeOffset(f32);

impl UiAction for NudgeOffset {
    fn execute(&self, world: &mut World) {
        let delta = self.0;
        world.resource_mut::<Mix>().offset += delta;
        let value = {
            let mut mix = world.resource_mut::<Mix>();
            mix.offset = mix.offset.clamp(-2.0, 2.0);
            mix.offset
        };
        // Push into the slider widget (programmatic set — no event echo).
        let mut sliders = world.query::<(&mut Slider, &Drives)>();
        for (mut slider, drives) in sliders.iter_mut(world) {
            if matches!(drives, Drives::Offset) {
                slider.set(value);
            }
        }
    }
}

/// Reset everything to defaults via programmatic writes.
struct ResetAction;

impl UiAction for ResetAction {
    fn execute(&self, world: &mut World) {
        *world.resource_mut::<Mix>() = Mix::default();
        let defaults = Mix::default();
        let mut sliders = world.query::<(&mut Slider, &Drives)>();
        for (mut slider, drives) in sliders.iter_mut(world) {
            match drives {
                Drives::Channel(i) => slider.set(defaults.rgb[*i]),
                Drives::Offset => slider.set(defaults.offset),
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn(Node::centered(24.0)).with_children(|root| {
        root.ui_text(TextRole::Heading, "Slider — RGB mixer");

        // Color swatch driven by the three channel sliders.
        root.spawn((
            Node {
                width: Val::Px(220.0),
                height: Val::Px(60.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(90, 140, 200)),
            BorderColor::all(Color::srgb(0.35, 0.35, 0.4)),
            Swatch,
        ));

        // R / G / B rows: label + slider (0..255, snap 1) + value.
        let channels = [("R", 90.0), ("G", 140.0), ("B", 200.0)];
        for (i, (name, initial)) in channels.into_iter().enumerate() {
            root.spawn(Node::row(10.0)).with_children(|row| {
                row.ui_text(TextRole::Body, name);
                let slider = row.spawn_slider(SliderConfig {
                    width: Val::Px(220.0),
                    min: 0.0,
                    max: 255.0,
                    initial,
                    step: 1.0,
                    ..default()
                });
                row.commands().entity(slider).insert(Drives::Channel(i));
                // Fixed width so 1→3 digit swings don't reflow the row.
                row.ui_text(TextRole::Body, format!("{initial:.0}"))
                    .insert((
                        Node {
                            width: Val::Px(36.0),
                            ..default()
                        },
                        ChannelLabel(i),
                    ));
            });
        }

        // Tuning-panel row: slider for feel, −/+ buttons for precision.
        root.ui_text(TextRole::Button, "Offset (slider + fine buttons)");
        root.spawn(Node::row(10.0)).with_children(|row| {
            row.spawn_button(NudgeOffset(-0.05), "-");
            let slider = row.spawn_slider(SliderConfig::symmetric(2.0, 0.0, 0.0));
            row.commands().entity(slider).insert(Drives::Offset);
            row.spawn_button(NudgeOffset(0.05), "+");
            row.ui_text(TextRole::Body, "+0.00").insert((
                Node {
                    width: Val::Px(52.0),
                    ..default()
                },
                OffsetLabel,
            ));
        });

        root.spawn_button(ResetAction, "Reset");
    });
}

/// SliderChanged → live state. One handler for all sliders; the Drives
/// marker on the slider root says what it edits.
fn apply_slider_changes(
    mut events: MessageReader<SliderChanged>,
    drives: Query<&Drives>,
    mut mix: ResMut<Mix>,
) {
    for event in events.read() {
        match drives.get(event.slider) {
            Ok(Drives::Channel(i)) => mix.rgb[*i] = event.value,
            Ok(Drives::Offset) => mix.offset = event.value,
            Err(_) => {}
        }
    }
}

fn sync_labels(
    mix: Res<Mix>,
    mut channel_labels: Query<(&ChannelLabel, &mut Text), Without<OffsetLabel>>,
    mut offset_labels: Query<&mut Text, With<OffsetLabel>>,
) {
    if !mix.is_changed() {
        return;
    }
    for (label, mut text) in &mut channel_labels {
        **text = format!("{:.0}", mix.rgb[label.0]);
    }
    for mut text in &mut offset_labels {
        **text = format!("{:+.2}", mix.offset);
    }
}

fn sync_swatch(mix: Res<Mix>, mut swatches: Query<&mut BackgroundColor, With<Swatch>>) {
    if !mix.is_changed() {
        return;
    }
    for mut bg in &mut swatches {
        *bg = BackgroundColor(Color::srgb_u8(
            mix.rgb[0] as u8,
            mix.rgb[1] as u8,
            mix.rgb[2] as u8,
        ));
    }
}
