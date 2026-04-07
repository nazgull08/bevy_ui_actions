use crate::core::{resolve_ui_theme, ButtonStyle, UiTheme};
use crate::interactions::{
    drag_system, handle_clicks, handle_hover_actions, handle_hover_exit_actions,
    handle_press_actions, handle_right_clicks, has_draggables, DragGhostStyle, DragState,
};
use crate::widgets::{
    clamp_scroll_bounds, handle_dialogue_dismiss_event, handle_dialogue_dismiss_input,
    handle_dismiss_event, handle_modal_dismiss, reveal_modal_panel, handle_scroll_input,
    handle_scrollbar_drag, handle_tab_clicks, handle_topic_panel_clicks, handle_track_click, topic_button_hover,
    apply_initial_visited_colors, has_dialogue, has_hypertext, has_scroll_views, hide_tooltip,
    hypertext_click, hypertext_hover, handle_topic_container,
    process_dialogue_queue, process_modal_queue,
    should_hide_tooltip,
    should_show_tooltip, show_tooltip, update_topic_panel, DialogueQueue, DialogueStyle,
    DismissDialogueEvent, DismissModalEvent, HyperLinkClicked, ListItemSelected, ModalQueue,
    ModalStyle, ScrollbarDragState, TopicDiscovered, TooltipSet, TooltipState, TooltipStyle,
    sync_active_tab_marker, sync_tab_content_visibility, update_border_visuals,
    update_interactive_visuals, update_progress_bars, update_scrollbar_thumb,
    update_tooltip_hover, update_visited_link_colors,
};
#[cfg(feature = "viewport3d")]
use crate::widgets::{
    has_viewports, viewport3d_cleanup, viewport3d_drag_rotate, viewport3d_track,
    Viewport3dDragState, Viewport3dTracked,
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
            .init_resource::<DialogueQueue>()
            .init_resource::<DialogueStyle>()
            .add_event::<ListItemSelected>()
            .add_event::<DismissModalEvent>()
            .add_event::<DismissDialogueEvent>()
            .add_event::<HyperLinkClicked>()
            .add_event::<TopicDiscovered>()
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
                    // Interactions
                    (
                        handle_clicks,
                        handle_right_clicks,
                        handle_hover_actions,
                        handle_hover_exit_actions,
                        handle_press_actions,
                        drag_system.run_if(has_draggables),
                    ),
                    // Tooltips
                    (
                        update_tooltip_hover.in_set(TooltipSet::DetectHover),
                        show_tooltip
                            .run_if(should_show_tooltip)
                            .in_set(TooltipSet::Display),
                        hide_tooltip
                            .run_if(should_hide_tooltip)
                            .in_set(TooltipSet::Display),
                    ),
                    // Theme + visuals
                    (
                        resolve_ui_theme,
                        update_interactive_visuals,
                        update_border_visuals,
                        update_progress_bars,
                    ),
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
                    (
                        handle_tab_clicks,
                        sync_tab_content_visibility,
                        sync_active_tab_marker,
                    ),
                    // Modal
                    (
                        process_modal_queue,
                        handle_modal_dismiss,
                        handle_dismiss_event.after(handle_modal_dismiss),
                    ),
                    // Hypertext + topic container
                    (
                        hypertext_click,
                        hypertext_hover,
                        apply_initial_visited_colors,
                        handle_topic_container.after(hypertext_click),
                        update_visited_link_colors.after(handle_topic_container),
                    )
                        .run_if(has_hypertext),
                    // Dialogue
                    (
                        process_dialogue_queue,
                        (
                            handle_dialogue_dismiss_input,
                            handle_dialogue_dismiss_event.after(handle_dialogue_dismiss_input),
                            handle_topic_panel_clicks,
                            topic_button_hover,
                            update_topic_panel.after(handle_topic_container),
                        )
                            .run_if(has_dialogue),
                    ),
                ),
            );

        // Modal: reveal after layout pass (prevents size pop on first frame).
        // Last schedule runs after PostUpdate (where Bevy computes UI layout),
        // so ComputedNode sizes are up to date when we check for stabilization.
        app.add_systems(Last, reveal_modal_panel);

        // Viewport3d (behind feature flag)
        #[cfg(feature = "viewport3d")]
        {
            app.init_resource::<Viewport3dDragState>()
                .init_resource::<Viewport3dTracked>()
                .add_systems(
                    Update,
                    (
                        viewport3d_track,
                        viewport3d_drag_rotate.run_if(has_viewports),
                        viewport3d_cleanup,
                    ),
                );
        }
    }
}
