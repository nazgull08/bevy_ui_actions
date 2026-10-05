//! Independent nested tabs, including buttons below twelve layout wrappers.
//! Run: `cargo run --example nested_tabs -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(12.0),
        padding: UiRect::all(Val::Px(12.0)),
        ..default()
    }
}

fn button(parent: &mut ChildSpawnerCommands, index: usize, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.22, 0.2)),
            Tab::new(index),
            VisualStyle::tab(),
            InteractiveVisual,
            Interaction::None,
        ))
        .with_children(|p| {
            p.ui_text(TextRole::Button, label);
        });
}

fn nested_group(parent: &mut ChildSpawnerCommands, name: &str, active: usize) {
    parent
        .spawn((column(), TabGroup::new(active)))
        .with_children(|group| {
            group.ui_text(TextRole::Heading, name);
            deep_buttons(group, 12);
            for (index, label) in [(0, "Equipment"), (1, "Bag")] {
                group
                    .spawn((column(), TabContent::new(index)))
                    .with_children(|p| {
                        p.ui_text(TextRole::Body, format!("{name}: {label}"));
                    });
            }
        });
}

fn deep_buttons(parent: &mut ChildSpawnerCommands, depth: usize) {
    if depth == 0 {
        parent.spawn(Node::default()).with_children(|row| {
            button(row, 0, "Equipment");
            button(row, 1, "Bag");
        });
    } else {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|p| deep_buttons(p, depth - 1));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((column(), TabGroup::new(0))).with_children(|outer| {
        outer.ui_text(TextRole::Heading, "Nested tabs: independent owners");
        outer.ui_text(TextRole::Caption, "Switch owner tabs independently. Open Journal and return: both owner choices must remain unchanged. Buttons have twelve layout ancestors.");
        outer.spawn(Node::default()).with_children(|row| {
            button(row, 0, "Inventory");
            button(row, 1, "Journal");
        });
        outer.spawn((Node::default(), TabContent::new(0))).with_children(|panels| {
            nested_group(panels, "Player (starts on Equipment)", 0);
            nested_group(panels, "Other owner (starts on Bag)", 1);
        });
        outer.spawn((column(), TabContent::new(1))).with_children(|p| {
            p.ui_text(TextRole::Body, "Journal: switching this outer tab must preserve inner choices.");
        });
    });
}
