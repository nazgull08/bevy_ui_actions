//! Run: cargo run --example scroll_bounds
//! Resize; use wheel on vertical/wrapped panels, drag/click their scrollbars.
//! 1/2 = UiScale 100/150%; H = hide/show wrapped panel (offset must survive).
//! Left/Right = horizontal scroll (a trackpad's horizontal wheel also works).
//! End = pin visible panels to bottom; R = remove last tiles (bounds must shrink).
use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

#[derive(Component, Clone, Copy, PartialEq)]
enum Demo {
    Vertical,
    Wrapped,
    Horizontal,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiActionsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (configure, controls).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|root| {
            root.ui_text(
                TextRole::Body,
                "Scroll bounds: 1/2 scale, H hide, arrows horizontal, End bottom, R remove",
            );
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (kind, label) in [
                    (Demo::Vertical, "Column + padding/gaps/margins"),
                    (Demo::Wrapped, "Direct children wrapping"),
                    (Demo::Horizontal, "Horizontal row / arrows"),
                ] {
                    row.spawn(Node {
                        width: Val::Px(260.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|panel| {
                        panel.ui_text(TextRole::Body, label);
                        panel.spawn_scroll_view_with(
                            ScrollViewConfig {
                                width: Val::Percent(100.0),
                                height: Val::Px(240.0),
                                direction: if kind == Demo::Horizontal {
                                    ScrollDirection::Horizontal
                                } else {
                                    ScrollDirection::Vertical
                                },
                                show_scrollbar: kind != Demo::Horizontal,
                                ..default()
                            },
                            |body| {
                                for i in 0..24 {
                                    body.spawn((
                                        kind,
                                        Node {
                                            width: if kind == Demo::Vertical {
                                                Val::Percent(100.0)
                                            } else {
                                                Val::Px(92.0)
                                            },
                                            height: Val::Px(52.0),
                                            flex_shrink: 0.0,
                                            margin: UiRect::all(Val::Px(3.0)),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        BackgroundColor(if i == 23 {
                                            Color::srgb(0.45, 0.3, 0.12)
                                        } else {
                                            Color::srgb(0.12, 0.22, 0.25)
                                        }),
                                    ))
                                    .with_children(|tile| {
                                        tile.ui_text(TextRole::Body, format!("Tile {}", i + 1));
                                    });
                                }
                            },
                        );
                    });
                }
            });
        });
}

fn configure(mut views: Query<(&Children, &mut Node), Added<ScrollView>>, kinds: Query<&Demo>) {
    for (children, mut node) in &mut views {
        let Some(kind) = children.iter().find_map(|child| kinds.get(child).ok()) else {
            continue;
        };
        node.padding = UiRect::all(Val::Px(14.0));
        node.row_gap = Val::Px(11.0);
        node.column_gap = Val::Px(9.0);
        if *kind == Demo::Wrapped {
            node.flex_direction = FlexDirection::Row;
            node.flex_wrap = FlexWrap::Wrap;
            node.align_content = AlignContent::FlexStart;
        }
    }
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut scale: ResMut<UiScale>,
    mut commands: Commands,
    mut views: Query<(Entity, &Children, &mut Node, &mut ScrollPosition), With<ScrollView>>,
    kinds: Query<&Demo>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        scale.0 = 1.0;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        scale.0 = 1.5;
    }
    for (entity, children, mut node, mut pos) in &mut views {
        let Some(kind) = children.iter().find_map(|child| kinds.get(child).ok()) else {
            continue;
        };
        if keys.just_pressed(KeyCode::KeyH) && *kind == Demo::Wrapped {
            node.display = if node.display == Display::None {
                Display::Flex
            } else {
                Display::None
            };
        }
        if *kind == Demo::Horizontal {
            if keys.just_pressed(KeyCode::ArrowRight) {
                pos.0.x += 80.0;
            }
            if keys.just_pressed(KeyCode::ArrowLeft) {
                pos.0.x -= 80.0;
            }
        }
        if keys.just_pressed(KeyCode::End) {
            commands.entity(entity).insert(StickToBottom::default());
        }
        if keys.just_pressed(KeyCode::KeyR)
            && let Some(last) = children.iter().last()
        {
            commands.entity(last).despawn();
        }
    }
}
