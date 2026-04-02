mod dialogue;
mod hypertext;
mod list_view;
mod modal;
mod panel;
mod progress_bar;
mod scroll_view;
mod tabs;
mod tooltip;
#[cfg(feature = "viewport3d")]
mod viewport3d;
mod visual;

pub use progress_bar::{ProgressBar, ProgressBarConfig, ProgressBarFill, SpawnProgressBarExt};
pub use tabs::{Tab, TabContent, TabGroup};
pub use tooltip::{
    StatDiff, Tooltip, TooltipBuilder, TooltipContent, TooltipSection, TooltipSet, TooltipState,
    TooltipStyle, TooltipUI,
};
pub use list_view::{
    ListItem, ListItemSelected, ListView, ListViewConfig, ListViewItems, SelectionMode,
    SpawnListViewExt,
};
pub use modal::{
    DismissModal, DismissModalEvent, Modal, ModalBackdrop, ModalPanel, ModalQueue, ModalRequest,
    ModalStyle, spawn_modal_button,
};
pub use panel::{PanelConfig, SpawnPanelExt};
pub use scroll_view::{
    ScrollDirection, ScrollView, ScrollViewConfig, ScrollbarDragState, ScrollbarThumb,
    ScrollbarTrack, SpawnScrollViewExt,
};
pub use visual::{Active, BorderStyle, Disabled, InteractiveVisual, Selected, VisualStyle};

// Re-export systems for plugin
pub(crate) use progress_bar::update_progress_bars;
pub(crate) use tabs::{handle_tab_clicks, sync_active_tab_marker, sync_tab_content_visibility};
pub(crate) use tooltip::{
    hide_tooltip, should_hide_tooltip, should_show_tooltip, show_tooltip, update_tooltip_hover,
};
pub(crate) use scroll_view::{
    clamp_scroll_bounds, handle_scroll_input, handle_scrollbar_drag, handle_track_click,
    has_scroll_views, update_scrollbar_thumb,
};
pub(crate) use modal::{handle_dismiss_event, handle_modal_dismiss, process_modal_queue};
pub use dialogue::{
    DialogueBox, DialogueConfig, DialogueContent, DialoguePosition, DialogueQueue,
    DialogueRequest, DialogueScroll, DialogueStyle, DialogueTopicButton, DialogueTopicPanel,
    DismissDialogue, DismissDialogueEvent, TopicDiscovered, TopicEntry, TopicRegistry,
    append_dialogue_text, has_dialogue,
};
pub(crate) use dialogue::{
    handle_dialogue_dismiss_event, handle_dialogue_dismiss_input, handle_dialogue_topic,
    handle_topic_panel_clicks, process_dialogue_queue, topic_button_hover, update_topic_panel,
};
pub use hypertext::{
    HyperLinkClicked, HyperLinkSpan, HyperText, HyperTextConfig, HyperTextHoverState,
    SpawnHyperTextExt,
};
pub(crate) use hypertext::{apply_initial_visited_colors, has_hypertext, hypertext_click, hypertext_hover, update_visited_link_colors};
#[cfg(feature = "viewport3d")]
pub use viewport3d::{
    Viewport3d, Viewport3dCamera, Viewport3dConfig, Viewport3dDragState, Viewport3dHandle,
    Viewport3dPivot, Viewport3dRotation, SpawnViewport3dExt,
};
#[cfg(feature = "viewport3d")]
pub(crate) use viewport3d::{
    has_viewports, viewport3d_cleanup, viewport3d_drag_rotate, viewport3d_track,
    Viewport3dTracked,
};
pub(crate) use visual::{update_border_visuals, update_interactive_visuals};
