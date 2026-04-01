//! Menu example with hover effects and tooltips.
//!
//! Demonstrates:
//! - State management via actions
//! - Tooltips on buttons
//! - UiTextExt for text spawning with roles
//!
//! Run: `cargo run --example menu -p bevy_ui_actions`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_status_text)
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Settings,
}

// ============ Click Actions ============

struct StartGameAction;

impl UiAction for StartGameAction {
    fn execute(&self, world: &mut World) {
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        info!("Starting game...");
    }
}

struct OpenSettingsAction;

impl UiAction for OpenSettingsAction {
    fn execute(&self, world: &mut World) {
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Settings);
        info!("Opening settings...");
    }
}

struct BackToMenuAction;

impl UiAction for BackToMenuAction {
    fn execute(&self, world: &mut World) {
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        info!("Back to menu...");
    }
}

struct QuitAction;

impl UiAction for QuitAction {
    fn execute(&self, world: &mut World) {
        world.send_event(AppExit::Success);
    }
}

// ============ UI Components ============

#[derive(Component)]
struct StatusText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node::centered(15.0))
        .with_children(|parent| {
            parent.ui_text(TextRole::Title, "Main Menu");

            parent
                .ui_text(TextRole::Heading, "State: Menu")
                .insert(StatusText);

            parent.ui_text(TextRole::Label, "Hover over buttons to see tooltips");

            // Spacer
            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Buttons with tooltips
            spawn_menu_button(
                parent,
                StartGameAction,
                "Start Game",
                "Begin your adventure!",
            );
            spawn_menu_button(
                parent,
                OpenSettingsAction,
                "Settings",
                "Configure game options",
            );
            spawn_menu_button(
                parent,
                BackToMenuAction,
                "Back to Menu",
                "Return to main menu",
            );
            spawn_menu_button(parent, QuitAction, "Quit", "Exit the game");
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    action: impl UiAction,
    label: &str,
    tooltip_text: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            OnClick::new(action),
            Tooltip::new(tooltip_text),
            InteractiveVisual,
        ))
        .with_children(|btn| {
            btn.ui_text(TextRole::Button, label);
        });
}

fn update_status_text(state: Res<State<GameState>>, mut query: Query<&mut Text, With<StatusText>>) {
    if state.is_changed() {
        for mut text in &mut query {
            **text = format!("State: {:?}", state.get());
        }
    }
}
