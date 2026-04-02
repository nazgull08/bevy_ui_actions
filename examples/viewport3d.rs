//! Viewport3d example — 3D model preview inside UI.
//!
//! Demonstrates:
//! - Spawning a 3D viewport widget with render-to-texture
//! - Adding 3D content (cube) as children of the pivot entity
//! - Drag-to-rotate interaction
//!
//! Run: `cargo run --example viewport3d -p bevy_ui_actions --features viewport3d`

use bevy::prelude::*;
use bevy_ui_actions::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiActionsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_cube)
        .run();
}

#[derive(Component)]
struct AutoRotate;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Main scene camera (for the UI)
    commands.spawn(Camera2d);

    // --- Viewport 1: interactive (drag to rotate) ---
    let config1 = Viewport3dConfig {
        size: UVec2::new(300, 300),
        render_layer: 1,
        camera_distance: 4.0,
        camera_height: 1.0,
        rotatable: true,
        ..default()
    };
    let handle1 = commands.spawn_viewport3d(&config1, &mut images);

    // Spawn a cube on the pivot (layer 1)
    commands.entity(handle1.pivot).with_children(|pivot| {
        pivot.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.3, 0.2),
                ..default()
            })),
            Transform::IDENTITY,
            handle1.render_layer.clone(),
        ));
    });

    // --- Viewport 2: auto-rotating sphere ---
    let config2 = Viewport3dConfig {
        size: UVec2::new(200, 200),
        render_layer: 2,
        camera_distance: 3.0,
        camera_height: 0.5,
        rotatable: false,
        background: Color::linear_rgba(0.05, 0.08, 0.12, 1.0),
        light_intensity: 3000.0,
        ..default()
    };
    let handle2 = commands.spawn_viewport3d(&config2, &mut images);

    commands.entity(handle2.pivot).with_children(|pivot| {
        pivot.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.6, 0.9),
                ..default()
            })),
            Transform::IDENTITY,
            handle2.render_layer.clone(),
            AutoRotate,
        ));
    });

    // --- UI layout ---
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|root| {
            root.ui_text(TextRole::Heading, "Viewport3d Demo");

            // Row with two viewports
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                // Left: interactive viewport
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|col| {
                    col.ui_text(TextRole::Label, "Drag to rotate");
                    col.spawn(Node::default())
                        .add_child(handle1.ui_entity);
                });

                // Right: auto-rotating viewport
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|col| {
                    col.ui_text(TextRole::Label, "Auto-rotate");
                    col.spawn(Node::default())
                        .add_child(handle2.ui_entity);
                });
            });
        });
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<AutoRotate>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.0);
        transform.rotate_x(time.delta_secs() * 0.3);
    }
}
