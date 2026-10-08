//! Decorative, antialiased UI strokes in the parent's logical pixel coordinates.
//! The material is embedded: consumers need no shader files in their asset directory.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// Maximum point count after rounded corners have been tessellated.
pub const MAX_UI_POLYLINE_POINTS: usize = 64;
const CORNER_STEPS: usize = 6;

/// Appearance of a decorative stroke, measured in logical UI pixels.
#[derive(Clone, Debug)]
pub struct UiStrokeStyle {
    pub color: Color,
    pub width: f32,
    /// Distance trimmed from each adjacent segment for a quadratic corner.
    /// Clamped to half the segment lengths. Zero keeps the original path.
    pub corner_radius: f32,
    /// An optional filled dot at the last point. Zero disables it.
    pub end_dot_radius: f32,
}

impl Default for UiStrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            width: 1.0,
            corner_radius: 0.0,
            end_dot_radius: 0.0,
        }
    }
}

/// A noninteractive polyline with round caps and optional rounded bends.
///
/// Spawn under a UI node with a defined size. Points are relative to that
/// parent's top-left content origin, in logical pixels. The widget owns its
/// Node's position, dimensions and display, and its generated MaterialNode.
/// Parent overflow/clipping, visibility and ZIndex work as for ordinary UI.
/// Mutate this component in Update or before [`UiPolylineSet::Prepare`] in PostUpdate.
/// Despawn the entity to remove the stroke.
///
/// Nonfinite input, nonpositive width, fewer than two distinct points or a
/// tessellated path exceeding [`MAX_UI_POLYLINE_POINTS`] hides the stroke.
/// An invalid path never displays a truncated connection.
#[derive(Component, Clone, Debug)]
#[require(Node)]
pub struct UiPolyline {
    pub points: Vec<Vec2>,
    pub style: UiStrokeStyle,
}

impl UiPolyline {
    pub fn new(points: impl Into<Vec<Vec2>>) -> Self {
        Self {
            points: points.into(),
            style: UiStrokeStyle::default(),
        }
    }

    pub fn with_style(mut self, style: UiStrokeStyle) -> Self {
        self.style = style;
        self
    }
}

/// Runs in PostUpdate; writes the stroke material and layout before UI Layout.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UiPolylineSet {
    Prepare,
}

#[derive(Clone, Debug, ShaderType)]
struct StrokeUniform {
    color: Vec4,
    // Logical bounds width/height, half stroke width, end dot radius.
    metrics: Vec4,
    // x is the number of points; remaining components are reserved/alignment.
    count: UVec4,
    points: [Vec4; MAX_UI_POLYLINE_POINTS],
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(crate) struct UiPolylineMaterial {
    #[uniform(0)]
    stroke: StrokeUniform,
}

impl UiMaterial for UiPolylineMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_ui_actions/widgets/polyline.wgsl".into()
    }
}

pub(crate) fn register_polylines(app: &mut App) {
    embedded_asset!(app, "polyline.wgsl");
    app.add_plugins(UiMaterialPlugin::<UiPolylineMaterial>::default())
        .add_systems(
            PostUpdate,
            prepare_polylines
                .in_set(UiPolylineSet::Prepare)
                .before(bevy::ui::UiSystems::Layout),
        );
}

fn prepare_polylines(
    mut commands: Commands,
    mut materials: ResMut<Assets<UiPolylineMaterial>>,
    lines: Query<
        (
            Entity,
            &UiPolyline,
            Option<&MaterialNode<UiPolylineMaterial>>,
        ),
        Changed<UiPolyline>,
    >,
) {
    for (entity, line, material_node) in &lines {
        let Some((node, stroke)) = prepare_stroke(line) else {
            commands.entity(entity).insert((
                Node {
                    display: Display::None,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            continue;
        };
        if let Some(mut material) = material_node.and_then(|node| materials.get_mut(&node.0)) {
            material.stroke = stroke;
            commands.entity(entity).insert((node, Pickable::IGNORE));
        } else {
            commands.entity(entity).insert((
                node,
                Pickable::IGNORE,
                MaterialNode(materials.add(UiPolylineMaterial { stroke })),
            ));
        }
    }
}

fn prepare_stroke(line: &UiPolyline) -> Option<(Node, StrokeUniform)> {
    let style = &line.style;
    if !style.width.is_finite()
        || style.width <= 0.0
        || !style.corner_radius.is_finite()
        || style.corner_radius < 0.0
        || !style.end_dot_radius.is_finite()
        || style.end_dot_radius < 0.0
        || line.points.len() > MAX_UI_POLYLINE_POINTS
        || line.points.iter().any(|p| !p.is_finite())
    {
        return None;
    }
    let mut distinct = Vec::with_capacity(line.points.len());
    for &point in &line.points {
        if distinct
            .last()
            .is_none_or(|last: &Vec2| last.distance_squared(point) > 1e-6)
        {
            distinct.push(point);
        }
    }
    if distinct.len() < 2 {
        return None;
    }
    let path = rounded_path(&distinct, style.corner_radius);
    if path.len() > MAX_UI_POLYLINE_POINTS {
        return None;
    }
    let padding = (style.width * 0.5).max(style.end_dot_radius) + 2.0;
    let min = path
        .iter()
        .fold(Vec2::splat(f32::INFINITY), |a, &b| a.min(b))
        - Vec2::splat(padding);
    let max = path
        .iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), |a, &b| a.max(b))
        + Vec2::splat(padding);
    let size = max - min;
    if !size.is_finite() || !min.is_finite() || size.cmple(Vec2::ZERO).any() {
        return None;
    }
    let mut points = [Vec4::ZERO; MAX_UI_POLYLINE_POINTS];
    for (target, &point) in points.iter_mut().zip(&path) {
        *target = (point - min).extend(0.0).extend(0.0);
    }
    Some((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(min.x),
            top: Val::Px(min.y),
            width: Val::Px(size.x),
            height: Val::Px(size.y),
            min_width: Val::Px(0.0),
            min_height: Val::Px(0.0),
            flex_shrink: 0.0,
            ..default()
        },
        StrokeUniform {
            color: style.color.to_linear().to_vec4(),
            metrics: Vec4::new(size.x, size.y, style.width * 0.5, style.end_dot_radius),
            count: UVec4::new(path.len() as u32, 0, 0, 0),
            points,
        },
    ))
}

fn rounded_path(points: &[Vec2], radius: f32) -> Vec<Vec2> {
    if radius == 0.0 {
        return points.to_vec();
    }
    let mut path = Vec::with_capacity(2 + (points.len() - 2) * (CORNER_STEPS + 1));
    path.push(points[0]);
    for window in points.windows(3) {
        let [previous, corner, next] = [window[0], window[1], window[2]];
        let incoming = previous - corner;
        let outgoing = next - corner;
        let trim = radius
            .min(incoming.length() * 0.5)
            .min(outgoing.length() * 0.5);
        let before = corner + incoming.normalize_or_zero() * trim;
        let after = corner + outgoing.normalize_or_zero() * trim;
        path.push(before);
        for step in 1..=CORNER_STEPS {
            let t = step as f32 / CORNER_STEPS as f32;
            path.push(before.lerp(corner, t).lerp(corner.lerp(after, t), t));
        }
    }
    path.push(*points.last().unwrap());
    path
}
