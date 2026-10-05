//! Horizontal value slider: track + draggable thumb.
//!
//! Drag the thumb (or click anywhere on the track) to set a value in
//! `min..=max`, optionally snapped to `step`. Every user-driven change
//! emits [`SliderChanged`]; setting [`Slider::value`] from code (e.g.
//! loading a preset) moves the thumb without emitting, so programmatic
//! writes never echo back as events.
//!
//! Built for tuning panels (pose editors, debug consoles, settings)
//! where a value needs continuous mouse adjustment. Pair with buttons
//! for precise stepping — see `examples/slider.rs`.

use crate::core::{UiInputScope, is_in_scope};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

// ============================================================
// Types
// ============================================================

/// Slider state, lives on the widget root (the track node).
///
/// Mutate [`Slider::value`] (or call [`Slider::set`]) to update the
/// widget from code — the thumb syncs every frame. Only mouse
/// interaction emits [`SliderChanged`].
#[derive(Component)]
pub struct Slider {
    /// Current value, always within `min..=max`.
    pub value: f32,
    /// Lower bound.
    pub min: f32,
    /// Upper bound.
    pub max: f32,
    /// Snap increment. `0.0` = continuous.
    pub step: f32,
}

impl Slider {
    /// Clamp (and snap, if `step` is set) then assign.
    pub fn set(&mut self, value: f32) {
        self.value = self.quantize(value);
    }

    /// Current position as a `0..=1` ratio of the range.
    pub fn ratio(&self) -> f32 {
        if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.0
        }
    }

    fn quantize(&self, value: f32) -> f32 {
        let snapped = if self.step > 0.0 {
            self.min + ((value - self.min) / self.step).round() * self.step
        } else {
            value
        };
        snapped.clamp(self.min, self.max)
    }
}

/// Marker on the thumb node. Points to the slider root.
#[derive(Component)]
pub struct SliderThumb {
    /// The slider root this thumb belongs to.
    pub slider: Entity,
}

/// Marker on the fill node (colored strip left of the thumb).
#[derive(Component)]
pub struct SliderFill;

/// Event: a slider's value changed through mouse interaction.
#[derive(Message)]
pub struct SliderChanged {
    /// The slider root entity.
    pub slider: Entity,
    /// The new value.
    pub value: f32,
}

/// Global drag state for slider thumb dragging.
#[derive(Resource, Default)]
pub struct SliderDragState {
    /// Which slider root is being dragged.
    pub dragging: Option<Entity>,
}

// ============================================================
// Config
// ============================================================

/// Configuration for spawning a slider.
#[derive(Clone, Debug)]
pub struct SliderConfig {
    /// Track width.
    pub width: Val,
    /// Track height (the clickable strip).
    pub height: Val,
    /// Lower bound.
    pub min: f32,
    /// Upper bound.
    pub max: f32,
    /// Starting value.
    pub initial: f32,
    /// Snap increment. `0.0` = continuous.
    pub step: f32,
    /// Track background color.
    pub track_color: Color,
    /// Fill strip color left of the thumb; `None` = no fill.
    pub fill_color: Option<Color>,
    /// Thumb color.
    pub thumb_color: Color,
    /// Thumb width in pixels (height follows the track).
    pub thumb_width: f32,
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            width: Val::Px(160.0),
            height: Val::Px(14.0),
            min: 0.0,
            max: 1.0,
            initial: 0.0,
            step: 0.0,
            track_color: Color::srgb(0.15, 0.15, 0.18),
            fill_color: Some(Color::srgb(0.3, 0.4, 0.55)),
            thumb_color: Color::srgb(0.6, 0.6, 0.68),
            thumb_width: 10.0,
        }
    }
}

impl SliderConfig {
    /// Symmetric range around zero: `-half..=half`.
    pub fn symmetric(half: f32, initial: f32, step: f32) -> Self {
        Self {
            min: -half,
            max: half,
            initial,
            step,
            ..default()
        }
    }

    /// Angle preset: `-180..=180`, snapped to whole degrees.
    pub fn degrees(initial: f32) -> Self {
        Self {
            min: -180.0,
            max: 180.0,
            initial,
            step: 1.0,
            ..default()
        }
    }
}

// ============================================================
// Spawn Extension
// ============================================================

/// Extension trait for spawning a slider.
pub trait SpawnSliderExt {
    /// Spawn a slider; returns the root entity (holds [`Slider`]).
    ///
    /// Attach your own marker to the returned entity to identify it when
    /// handling [`SliderChanged`].
    fn spawn_slider(&mut self, config: SliderConfig) -> Entity;
}

impl SpawnSliderExt for ChildSpawnerCommands<'_> {
    fn spawn_slider(&mut self, config: SliderConfig) -> Entity {
        let slider = Slider {
            value: config.initial,
            min: config.min,
            max: config.max,
            step: config.step,
        };
        let ratio = slider.ratio();

        let mut root = self.spawn((
            Node {
                width: config.width,
                height: config.height,
                // PORT-0.18: BorderRadius is a Node field now, not a component.
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(config.track_color),
            Interaction::None,
            slider,
        ));
        let root_id = root.id();

        root.with_children(|track| {
            if let Some(fill_color) = config.fill_color {
                track.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(ratio * 100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    SliderFill,
                ));
            }
            track.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0), // positioned by update_slider_visuals
                    top: Val::Px(0.0),
                    width: Val::Px(config.thumb_width),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(config.thumb_color),
                Interaction::None,
                SliderThumb { slider: root_id },
            ));
        });

        root_id
    }
}

// ============================================================
// Systems
// ============================================================

/// Run condition: any slider exists.
pub(crate) fn has_sliders(query: Query<(), With<Slider>>) -> bool {
    !query.is_empty()
}

/// Press on the track or thumb starts a drag; while dragging, the value
/// follows the cursor with absolute mapping (thumb centers under the
/// cursor). Emits [`SliderChanged`] whenever the value actually changes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn slider_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag_state: ResMut<SliderDragState>,
    mut sliders: Query<(
        Entity,
        &mut Slider,
        &Interaction,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    thumbs: Query<(&Interaction, &SliderThumb)>,
    mut events: MessageWriter<SliderChanged>,
    scope: Option<Res<UiInputScope>>,
    parents: Query<&ChildOf>,
) {
    let cursor_x = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .map(|p| p.x);

    // Start drag: press over the track or its thumb.
    if mouse.just_pressed(MouseButton::Left) && drag_state.dragging.is_none() {
        let hovered_via_thumb = |slider_entity: Entity| {
            thumbs.iter().any(|(interaction, thumb)| {
                thumb.slider == slider_entity
                    && matches!(interaction, Interaction::Pressed | Interaction::Hovered)
            })
        };

        for (entity, _, interaction, _, _) in &sliders {
            let over_track = matches!(interaction, Interaction::Pressed | Interaction::Hovered);
            if !over_track && !hovered_via_thumb(entity) {
                continue;
            }
            if let Some(ref scope) = scope
                && !is_in_scope(entity, scope, &parents)
            {
                continue;
            }
            drag_state.dragging = Some(entity);
            break;
        }
    }

    // Continue / finish drag.
    let Some(dragged) = drag_state.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        drag_state.dragging = None;
        return;
    }
    let Some(cursor_x) = cursor_x else {
        return;
    };
    let Ok((entity, mut slider, _, transform, node)) = sliders.get_mut(dragged) else {
        drag_state.dragging = None;
        return;
    };

    // ComputedNode/GlobalTransform are physical px, the cursor is logical.
    let scale = node.inverse_scale_factor();
    let track_width = node.size().x * scale;
    if track_width <= 0.0 {
        return;
    }
    let track_left = transform.translation.x * scale - track_width / 2.0;
    let ratio = ((cursor_x - track_left) / track_width).clamp(0.0, 1.0);
    let target = slider.min + ratio * (slider.max - slider.min);

    let previous = slider.value;
    slider.set(target);
    if slider.value != previous {
        events.write(SliderChanged {
            slider: entity,
            value: slider.value,
        });
    }
}

/// Sync thumb position and fill width from the slider value (covers both
/// mouse drags and programmatic writes).
#[allow(clippy::type_complexity)]
pub(crate) fn update_slider_visuals(
    sliders: Query<(&Slider, &ComputedNode, &Children)>,
    mut thumbs: Query<(&mut Node, &ComputedNode), (With<SliderThumb>, Without<SliderFill>)>,
    mut fills: Query<&mut Node, (With<SliderFill>, Without<SliderThumb>)>,
) {
    for (slider, track_node, children) in &sliders {
        let track_width = track_node.size().x;
        if track_width <= 0.0 {
            continue;
        }
        let ratio = slider.ratio();

        for child in children.iter() {
            if let Ok((mut thumb_node, thumb_computed)) = thumbs.get_mut(child) {
                let thumb_width = thumb_computed.size().x;
                let usable = (track_width - thumb_width).max(0.0);
                // ComputedNode sizes are physical px; Node.left is logical.
                let scale = track_node.inverse_scale_factor();
                thumb_node.left = Val::Px(ratio * usable * scale);
            } else if let Ok(mut fill_node) = fills.get_mut(child) {
                fill_node.width = Val::Percent(ratio * 100.0);
            }
        }
    }
}
