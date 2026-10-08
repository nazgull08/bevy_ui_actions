//! Rounded strokes, antialiasing, clipping, updates and click-through.
//! Run: cargo run --example polyline

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

#[derive(Resource, Default)]
struct DemoState {
    paused: bool,
    highlighted: bool,
    clicks: u32,
}

#[derive(Component)]
struct MovingLeader;

#[derive(Component)]
struct ClickCounter;

struct CountClick;
impl UiAction for CountClick {
    fn execute(&self, world: &mut World) {
        world.resource_mut::<DemoState>().clicks += 1;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .init_resource::<DemoState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_demo).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|root| {
            root.ui_text_styled(
                "Space: pause | H: highlight | +/-: UI scale",
                18.0,
                Color::WHITE,
            );
            root.spawn((
                Node {
                    width: Val::Px(680.0),
                    height: Val::Px(400.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.055, 0.065, 0.07)),
            ))
            .with_children(|canvas| {
                canvas.spawn((
                    UiPolyline::new(vec![
                        Vec2::new(70.0, 80.0),
                        Vec2::new(190.0, 80.0),
                        Vec2::new(500.0, 150.0),
                    ])
                    .with_style(UiStrokeStyle {
                        color: Color::srgba(0.65, 0.72, 0.69, 0.65),
                        width: 1.0,
                        corner_radius: 12.0,
                        end_dot_radius: 3.0,
                    }),
                    MovingLeader,
                ));
                // The line crosses a button, but must not block it.
                let button = canvas.spawn_button(CountClick, "Click through the line");
                canvas.commands().entity(button).insert(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(270.0),
                    top: Val::Px(205.0),
                    width: Val::Px(240.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                });
                canvas.spawn((
                    UiPolyline::new(vec![Vec2::new(40.0, 230.0), Vec2::new(630.0, 230.0)])
                        .with_style(UiStrokeStyle {
                            color: Color::srgba(0.9, 0.5, 0.2, 0.8),
                            width: 2.0,
                            ..default()
                        }),
                    ZIndex(2),
                ));
                // Both ends cross the canvas bounds: parent clips them.
                canvas.spawn(
                    UiPolyline::new(vec![
                        Vec2::new(-70.0, 360.0),
                        Vec2::new(170.0, 320.0),
                        Vec2::new(540.0, 360.0),
                        Vec2::new(740.0, 310.0),
                    ])
                    .with_style(UiStrokeStyle {
                        color: Color::srgb(0.4, 0.6, 0.8),
                        width: 1.5,
                        corner_radius: 10.0,
                        ..default()
                    }),
                );
            });
            root.ui_text_styled("Clicks: 0", 16.0, Color::WHITE)
                .insert(ClickCounter);
        });
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut scale: ResMut<UiScale>,
) {
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }
    if keys.just_pressed(KeyCode::KeyH) {
        state.highlighted = !state.highlighted;
    }
    if keys.just_pressed(KeyCode::Equal) {
        scale.0 = (scale.0 + 0.25).min(2.0);
    }
    if keys.just_pressed(KeyCode::Minus) {
        scale.0 = (scale.0 - 0.25).max(0.5);
    }
}

fn update_demo(
    time: Res<Time>,
    state: Res<DemoState>,
    mut leaders: Query<&mut UiPolyline, With<MovingLeader>>,
    mut counters: Query<&mut Text, With<ClickCounter>>,
) {
    for mut line in &mut leaders {
        if !state.paused {
            line.points[2] = Vec2::new(500.0, 150.0 + 45.0 * time.elapsed_secs().sin());
        }
        let color = if state.highlighted {
            Color::srgb(0.9, 0.75, 0.45)
        } else {
            Color::srgba(0.65, 0.72, 0.69, 0.65)
        };
        if line.style.color != color {
            line.style.color = color;
        }
    }
    if state.is_changed() {
        for mut text in &mut counters {
            text.0 = format!("Clicks: {}", state.clicks);
        }
    }
}
