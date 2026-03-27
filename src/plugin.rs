use crate::core::ButtonStyle;
use crate::interactions::{
    drag_system, handle_clicks, handle_hover_actions, handle_hover_exit_actions,
    handle_press_actions, handle_right_clicks, has_draggables, DragGhostStyle, DragState,
};
use crate::widgets::{
    handle_tab_clicks, hide_tooltip, should_hide_tooltip, should_show_tooltip, show_tooltip,
    sync_active_tab_marker, sync_tab_content_visibility, update_border_visuals,
    update_interactive_visuals, update_progress_bars, update_tooltip_hover, TooltipSet,
    TooltipState, TooltipStyle,
};
use bevy::prelude::*;

pub struct UiActionsPlugin;

impl Plugin for UiActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonStyle>()
            .init_resource::<DragState>()
            .init_resource::<DragGhostStyle>()
            .init_resource::<TooltipState>()
            .init_resource::<TooltipStyle>()
            // Configure tooltip system ordering
            .configure_sets(
                Update,
                (
                    TooltipSet::DetectHover,
                    TooltipSet::GenerateContent,
                    TooltipSet::Display,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    // Click actions
                    handle_clicks,
                    handle_right_clicks,
                    // Hover actions
                    handle_hover_actions,
                    handle_hover_exit_actions,
                    handle_press_actions,
                    // Drag
                    drag_system.run_if(has_draggables),
                    // Tooltip systems with proper ordering
                    update_tooltip_hover.in_set(TooltipSet::DetectHover),
                    show_tooltip
                        .run_if(should_show_tooltip)
                        .in_set(TooltipSet::Display),
                    hide_tooltip
                        .run_if(should_hide_tooltip)
                        .in_set(TooltipSet::Display),
                    // Visual feedback (background + border)
                    update_interactive_visuals,
                    update_border_visuals,
                    // Progress bars
                    update_progress_bars,
                    // Tabs
                    handle_tab_clicks,
                    sync_tab_content_visibility,
                    sync_active_tab_marker,
                ),
            );
    }
}
