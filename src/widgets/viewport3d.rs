use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    view::RenderLayers,
};

/// Configuration for spawning a 3D viewport widget.
#[derive(Clone, Debug)]
pub struct Viewport3dConfig {
    /// Render texture resolution.
    pub size: UVec2,
    /// RenderLayers layer index (1-31). Objects on this layer are visible only to this viewport's camera.
    pub render_layer: u8,
    /// Camera distance from origin.
    pub camera_distance: f32,
    /// Camera vertical field of view in degrees.
    pub camera_fov: f32,
    /// Camera look-at target.
    pub camera_target: Vec3,
    /// Camera vertical offset (added to target Y for the camera position).
    pub camera_height: f32,
    /// Background color of the viewport.
    pub background: Color,
    /// Whether drag-to-rotate is enabled on the pivot.
    pub rotatable: bool,
    /// Rotation sensitivity (degrees per pixel of drag).
    pub rotate_sensitivity: f32,
    /// Point light intensity.
    pub light_intensity: f32,
    /// Point light offset from origin.
    pub light_offset: Vec3,
}

impl Default for Viewport3dConfig {
    fn default() -> Self {
        Self {
            size: UVec2::new(256, 256),
            render_layer: 1,
            camera_distance: 3.0,
            camera_fov: 45.0,
            camera_target: Vec3::ZERO,
            camera_height: 0.5,
            background: Color::linear_rgba(0.08, 0.08, 0.10, 1.0),
            rotatable: true,
            rotate_sensitivity: 0.4,
            light_intensity: 2000.0,
            light_offset: Vec3::new(2.0, 3.0, 2.0),
        }
    }
}

/// Component on the UI node that displays the 3D viewport.
#[derive(Component)]
pub struct Viewport3d {
    /// The camera entity rendering this viewport.
    pub camera: Entity,
    /// The point light entity.
    pub light: Entity,
    /// The pivot entity (parent your 3D content to this).
    pub pivot: Entity,
    /// RenderLayers to apply to your 3D content.
    pub render_layer: RenderLayers,
    /// The render target image handle.
    pub image: Handle<Image>,
    /// Whether drag-to-rotate is enabled.
    pub rotatable: bool,
    /// Rotation sensitivity.
    pub rotate_sensitivity: f32,
}

/// Marker on the viewport's Camera3d entity.
#[derive(Component)]
pub struct Viewport3dCamera {
    /// Back-reference to the UI entity.
    pub ui_entity: Entity,
}

/// Marker on the pivot entity. Parent your 3D objects to this.
#[derive(Component)]
pub struct Viewport3dPivot {
    /// Back-reference to the UI entity.
    pub ui_entity: Entity,
}

/// Rotation state for drag-to-rotate viewports.
#[derive(Component, Default)]
pub struct Viewport3dRotation {
    pub yaw: f32,
    pub pitch: f32,
}

/// Drag state tracking for viewport rotation.
#[derive(Resource, Default)]
pub struct Viewport3dDragState {
    /// Which viewport UI entity is being dragged, if any.
    pub dragging: Option<Entity>,
    /// Previous cursor position.
    pub last_pos: Vec2,
}

/// Handle returned after spawning a viewport. Use `render_layer` and `pivot` to add 3D content.
pub struct Viewport3dHandle {
    /// The UI entity displaying the viewport image.
    pub ui_entity: Entity,
    /// Camera entity (on the render layer).
    pub camera: Entity,
    /// Point light entity (on the render layer).
    pub light: Entity,
    /// Pivot entity — parent your 3D objects as children of this.
    pub pivot: Entity,
    /// The render layer to put on your 3D objects.
    pub render_layer: RenderLayers,
    /// The render target image handle.
    pub image: Handle<Image>,
}

/// Extension trait for spawning Viewport3d widgets.
pub trait SpawnViewport3dExt {
    fn spawn_viewport3d(
        &mut self,
        config: &Viewport3dConfig,
        images: &mut Assets<Image>,
    ) -> Viewport3dHandle;
}

impl SpawnViewport3dExt for Commands<'_, '_> {
    fn spawn_viewport3d(
        &mut self,
        config: &Viewport3dConfig,
        images: &mut Assets<Image>,
    ) -> Viewport3dHandle {
        // 1. Create render target image
        let extent = Extent3d {
            width: config.size.x,
            height: config.size.y,
            ..default()
        };
        let mut image = Image::new_fill(
            extent,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage =
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
        let image_handle = images.add(image);

        let layer = RenderLayers::layer(config.render_layer as usize);

        // 2. Spawn pivot (empty transform, user parents 3D content here)
        let pivot = self
            .spawn((
                Transform::default(),
                Visibility::default(),
                Viewport3dRotation::default(),
                layer.clone(),
                Name::new("Viewport3d_Pivot"),
            ))
            .id();

        // 3. Spawn camera
        let camera_pos = config.camera_target
            + Vec3::new(0.0, config.camera_height, config.camera_distance);
        let camera = self
            .spawn((
                Camera3d::default(),
                Camera {
                    target: bevy::render::camera::RenderTarget::Image(image_handle.clone().into()),
                    order: -1,
                    clear_color: ClearColorConfig::Custom(config.background),
                    ..default()
                },
                Projection::Perspective(PerspectiveProjection {
                    fov: config.camera_fov.to_radians(),
                    ..default()
                }),
                Transform::from_translation(camera_pos)
                    .looking_at(config.camera_target, Vec3::Y),
                layer.clone(),
                Name::new("Viewport3d_Camera"),
            ))
            .id();

        // 4. Spawn light
        let light = self
            .spawn((
                PointLight {
                    intensity: config.light_intensity,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(config.light_offset),
                layer.clone(),
                Name::new("Viewport3d_Light"),
            ))
            .id();

        // 5. Spawn UI node with ImageNode
        let ui_entity = self
            .spawn((
                ImageNode::new(image_handle.clone()),
                Node {
                    width: Val::Px(config.size.x as f32),
                    height: Val::Px(config.size.y as f32),
                    ..default()
                },
                Viewport3d {
                    camera,
                    light,
                    pivot,
                    render_layer: layer.clone(),
                    image: image_handle.clone(),
                    rotatable: config.rotatable,
                    rotate_sensitivity: config.rotate_sensitivity,
                },
                Name::new("Viewport3d"),
            ))
            .id();

        // Set back-references
        self.entity(camera).insert(Viewport3dCamera { ui_entity });
        self.entity(pivot).insert(Viewport3dPivot { ui_entity });

        Viewport3dHandle {
            ui_entity,
            camera,
            light,
            pivot,
            render_layer: layer,
            image: image_handle,
        }
    }
}

// ── Systems ──

/// Handles drag-to-rotate on viewport widgets.
pub(crate) fn viewport3d_drag_rotate(
    mut drag: ResMut<Viewport3dDragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    viewports: Query<(Entity, &Viewport3d, &ComputedNode, &GlobalTransform)>,
    mut rotations: Query<&mut Viewport3dRotation>,
    mut transforms: Query<&mut Transform, With<Viewport3dPivot>>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        drag.dragging = None;
        return;
    };

    // Start drag
    if mouse_button.just_pressed(MouseButton::Left) && drag.dragging.is_none() {
        for (entity, viewport, computed, gtf) in &viewports {
            if !viewport.rotatable {
                continue;
            }
            let node_pos = gtf.translation().truncate();
            let size = computed.size();
            let half = size / 2.0;
            let min = node_pos - half;
            let max = node_pos + half;
            if cursor_pos.x >= min.x
                && cursor_pos.x <= max.x
                && cursor_pos.y >= min.y
                && cursor_pos.y <= max.y
            {
                drag.dragging = Some(entity);
                drag.last_pos = cursor_pos;
                break;
            }
        }
    }

    // Continue drag
    if let Some(dragging_entity) = drag.dragging {
        if mouse_button.pressed(MouseButton::Left) {
            let delta = cursor_pos - drag.last_pos;
            drag.last_pos = cursor_pos;

            if let Ok((_, viewport, _, _)) = viewports.get(dragging_entity) {
                let sensitivity = viewport.rotate_sensitivity;
                if let Ok(mut rotation) = rotations.get_mut(viewport.pivot) {
                    rotation.yaw += delta.x * sensitivity;
                    rotation.pitch += delta.y * sensitivity;

                    if let Ok(mut transform) = transforms.get_mut(viewport.pivot) {
                        *transform = Transform::from_rotation(
                            Quat::from_rotation_y(rotation.yaw.to_radians())
                                * Quat::from_rotation_x(rotation.pitch.to_radians()),
                        );
                    }
                }
            }
        } else {
            drag.dragging = None;
        }
    }
}

/// Cleanup 3D entities when a Viewport3d UI node is despawned.
/// Uses a tracking resource since RemovedComponents can't access component data.
pub(crate) fn viewport3d_cleanup(
    mut commands: Commands,
    mut removed: RemovedComponents<Viewport3d>,
    mut tracked: ResMut<Viewport3dTracked>,
) {
    for entity in removed.read() {
        if let Some(info) = tracked.map.remove(&entity) {
            if let Ok(mut ec) = commands.get_entity(info.camera) {
                ec.despawn();
            }
            if let Ok(mut ec) = commands.get_entity(info.light) {
                ec.despawn();
            }
            if let Ok(mut ec) = commands.get_entity(info.pivot) {
                ec.despawn();
            }
        }
    }
}

/// Tracks viewport 3D entities for cleanup when UI node is despawned.
#[derive(Resource, Default)]
pub struct Viewport3dTracked {
    pub(crate) map: HashMap<Entity, Viewport3dEntities>,
}

#[derive(Clone)]
pub(crate) struct Viewport3dEntities {
    pub camera: Entity,
    pub light: Entity,
    pub pivot: Entity,
}

/// Registers newly spawned viewports for tracking.
pub(crate) fn viewport3d_track(
    query: Query<(Entity, &Viewport3d), Added<Viewport3d>>,
    mut tracked: ResMut<Viewport3dTracked>,
) {
    for (entity, viewport) in &query {
        tracked.map.insert(entity, Viewport3dEntities {
            camera: viewport.camera,
            light: viewport.light,
            pivot: viewport.pivot,
        });
    }
}

/// Run condition: at least one Viewport3d exists.
pub(crate) fn has_viewports(q: Query<(), With<Viewport3d>>) -> bool {
    !q.is_empty()
}
