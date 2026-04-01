use bevy::prelude::*;

/// Extension trait for common [`Node`] layout patterns.
pub trait NodeExt {
    /// Horizontal row with column gap.
    fn row(gap: f32) -> Node;

    /// Vertical column with row gap.
    fn column(gap: f32) -> Node;

    /// Full-size container (width + height 100%).
    fn fill() -> Node;

    /// Full-size centered column with row gap.
    fn centered(gap: f32) -> Node;
}

impl NodeExt for Node {
    fn row(gap: f32) -> Node {
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(gap),
            ..default()
        }
    }

    fn column(gap: f32) -> Node {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(gap),
            ..default()
        }
    }

    fn fill() -> Node {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        }
    }

    fn centered(gap: f32) -> Node {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(gap),
            ..default()
        }
    }
}
