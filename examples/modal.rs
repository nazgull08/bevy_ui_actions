//! Modal example.
//!
//! Demonstrates:
//! - Confirm dialog (sugar via `ModalRequest::confirm`)
//! - Custom modal with arbitrary content and buttons
//! - Non-dismissable modal with single button
//! - ESC and backdrop click to dismiss
//!
//! Run: `cargo run --example modal -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct StatusText;

// ============ Setup ============

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node::centered(20.0))
        .with_children(|root| {
            root.ui_text(TextRole::Heading, "Modal Examples");

            // Status text
            root.ui_text(TextRole::Body, "Click a button to open a modal")
                .insert(StatusText);

            // Button row
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                ..default()
            })
            .with_children(|row| {
                spawn_button(row, "Confirm Dialog", ShowConfirmModal);
                spawn_button(row, "Custom Modal", ShowCustomModal);
                spawn_button(row, "Non-dismissable", ShowLockedModal);
            });

            root.ui_text(
                TextRole::Caption,
                "ESC or click backdrop to dismiss (unless non-dismissable)",
            );
        });
}

// ============ Actions ============

struct ShowConfirmModal;

impl UiAction for ShowConfirmModal {
    fn execute(&self, world: &mut World) {
        let mut queue = world.resource_mut::<ModalQueue>();
        queue.show(
            ModalRequest::confirm(
                "Delete Item?",
                "Are you sure you want to delete this item? This action cannot be undone.",
            )
            .with_confirm(SetStatus("Item deleted!"))
            .with_cancel(SetStatus("Cancelled.")),
        );
    }
}

struct ShowCustomModal;

impl UiAction for ShowCustomModal {
    fn execute(&self, world: &mut World) {
        let mut queue = world.resource_mut::<ModalQueue>();
        queue.show(
            ModalRequest::new(|panel, style| {
                panel.ui_text(TextRole::Heading, "Custom Content");
                panel.ui_text(TextRole::Body, "This modal has completely custom content.");
                panel.ui_text(TextRole::Body, "You can put anything here:");

                // A colored box
                panel
                    .spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.3, 0.5)),
                    ))
                    .with_children(|box_node| {
                        box_node.ui_text(TextRole::Button, "A fancy box");
                    });

                // Three buttons
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_modal_button(
                            row,
                            "Save",
                            style.confirm_color,
                            style.button_padding_x,
                            style.button_padding_y,
                            DismissModal(true),
                        );
                        spawn_modal_button(
                            row,
                            "Discard",
                            Color::srgb(0.35, 0.25, 0.1),
                            style.button_padding_x,
                            style.button_padding_y,
                            DismissModal(false),
                        );
                        spawn_modal_button(
                            row,
                            "Cancel",
                            style.cancel_color,
                            style.button_padding_x,
                            style.button_padding_y,
                            DismissModal(false),
                        );
                    });
            })
            .with_confirm(SetStatus("Saved!"))
            .with_cancel(SetStatus("Custom modal closed.")),
        );
    }
}

struct ShowLockedModal;

impl UiAction for ShowLockedModal {
    fn execute(&self, world: &mut World) {
        let mut queue = world.resource_mut::<ModalQueue>();
        queue.show(
            ModalRequest::new(|panel, style| {
                panel.ui_text(TextRole::Heading, "Terms of Service");
                panel.ui_text(
                    TextRole::Body,
                    "You must accept the terms to continue. ESC and backdrop click are disabled.",
                );

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_modal_button(
                            row,
                            "Accept",
                            style.confirm_color,
                            style.button_padding_x,
                            style.button_padding_y,
                            DismissModal(true),
                        );
                    });
            })
            .with_dismissable(false)
            .with_confirm(SetStatus("Terms accepted.")),
        );
    }
}

struct SetStatus(&'static str);

impl UiAction for SetStatus {
    fn execute(&self, world: &mut World) {
        let mut query = world.query_filtered::<&mut Text, With<StatusText>>();
        for mut text in query.iter_mut(world) {
            **text = self.0.to_string();
        }
    }
}

// ============ Button helper ============

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, action: impl UiAction) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.25, 0.35)),
            OnClick::new(action),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text(TextRole::Button, label);
        });
}
