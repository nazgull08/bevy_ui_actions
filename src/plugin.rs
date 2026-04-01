use crate::core::{resolve_ui_theme, ButtonStyle, UiTheme};
use crate::interactions::{
    drag_system, handle_clicks, handle_hover_actions, handle_hover_exit_actions,
    handle_press_actions, handle_right_clicks, has_draggables, DragGhostStyle, DragState,
};
use crate::widgets::{
    clamp_scroll_bounds, handle_dismiss_event, handle_modal_dismiss, handle_scroll_input,
    handle_scrollbar_drag, handle_tab_clicks, handle_track_click, has_scroll_views, hide_tooltip,
    process_modal_queue, should_hide_tooltip, should_show_tooltip, show_tooltip, ListItemSelected,
    ModalQueue, ModalStyle, sync_active_tab_marker, sync_tab_content_visibility,
    update_border_visuals, update_interactive_visuals, update_progress_bars,
    update_scrollbar_thumb, update_tooltip_hover, DismissModalEvent, ScrollbarDragState,
    TooltipSet, TooltipState, TooltipStyle,
};
use bevy::prelude::*;

pub struct UiActionsPlugin;

impl Plugin for UiActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .init_resource::<ButtonStyle>()
            .init_resource::<DragState>()
            .init_resource::<DragGhostStyle>()
            .init_resource::<TooltipState>()
            .init_resource::<TooltipStyle>()
            .init_resource::<ScrollbarDragState>()
            .init_resource::<ModalStyle>()
            .init_resource::<ModalQueue>()
            .add_event::<ListItemSelected>()
            .add_event::<DismissModalEvent>()
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
                    // Theme: resolve font on newly spawned text
                    resolve_ui_theme,
                    // Visual feedback (background + border)
                    update_interactive_visuals,
                    update_border_visuals,
                    // Progress bars
                    update_progress_bars,
                    // Scroll
                    (
                        handle_scroll_input,
                        handle_scrollbar_drag,
                        handle_track_click,
                        clamp_scroll_bounds,
                        update_scrollbar_thumb,
                    )
                        .chain()
                        .run_if(has_scroll_views),
                    // Tabs
                    handle_tab_clicks,
                    sync_tab_content_visibility,
                    sync_active_tab_marker,
                    // Modal
                    process_modal_queue,
                    handle_modal_dismiss,
                    handle_dismiss_event.after(handle_modal_dismiss),
                ),
            );
    }
}
