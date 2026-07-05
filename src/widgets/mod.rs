mod dialogue;
mod hypertext;
mod list_view;
mod modal;
mod panel;
mod progress_bar;
mod scroll_view;
mod slot;
mod tabs;
mod tooltip;
#[cfg(feature = "viewport3d")]
mod viewport3d;
mod visual;
mod window;

pub use slot::{dragged_slot, Slot, SlotGridConfig, SpawnSlotGridExt};

pub(crate) use window::{
    apply_window_z, cleanup_windows, register_windows, window_close_on_escape, window_close_system,
    window_focus_system, window_move_system,
};
pub use window::{
    SpawnWindowExt, UiWindow, WindowClosable, WindowCloseButton, WindowConfig, WindowContent,
    WindowDrag, WindowDragState, WindowFocused, WindowManager, WindowMovable, WindowTitleBar,
    WINDOW_Z_BASE,
};

pub use list_view::{
    ListItem, ListItemSelected, ListView, ListViewConfig, ListViewItems, SelectionMode,
    SpawnListViewExt,
};
pub use modal::{
    spawn_modal_button, DismissModal, DismissModalEvent, Modal, ModalBackdrop, ModalPanel,
    ModalQueue, ModalRequest, ModalStyle,
};
pub use panel::{PanelConfig, SpawnPanelExt};
pub use progress_bar::{ProgressBar, ProgressBarConfig, ProgressBarFill, SpawnProgressBarExt};
pub use scroll_view::{
    ScrollDirection, ScrollView, ScrollViewConfig, ScrollbarDragState, ScrollbarThumb,
    ScrollbarTrack, SpawnScrollViewExt, StickToBottom,
};
pub use tabs::{Tab, TabContent, TabGroup};
pub use tooltip::{
    StatDiff, Tooltip, TooltipBuilder, TooltipContent, TooltipSection, TooltipSet, TooltipState,
    TooltipStyle, TooltipUI,
};
pub use visual::{Active, BorderStyle, Disabled, InteractiveVisual, Selected, VisualStyle};

// Re-export systems for plugin
pub(crate) use dialogue::{
    apply_append_text, apply_set_choices, dismiss_on_close_request, handle_choice_clicks,
    handle_choice_hotkeys, handle_close_button_clicks, handle_dialogue_close_input,
    handle_dialogue_dismiss_event, handle_topic_panel_clicks, process_dialogue_queue,
    track_active_topic, update_choice_button_visuals, update_topic_button_colors,
    update_topic_panel,
};
pub use dialogue::{
    has_dialogue, ActiveTopic, AppendDialogueText, DialogueBox, DialogueChoice,
    DialogueChoiceButton, DialogueChoiceSelected, DialogueChoicesRow, DialogueCloseButton,
    DialogueCloseRequested, DialogueConfig, DialogueContent, DialoguePosition,
    DialoguePresentation, DialogueQueue, DialogueRequest, DialogueScroll, DialogueStyle,
    DialogueTopicButton, DialogueTopicPanel, DialogueTopicsLocked, DismissDialogue,
    DismissDialogueEvent, SetDialogueChoices, TopicDiscovered, TopicEntry, TopicRegistry,
};
pub use hypertext::{
    append_topic_block, HyperLinkClicked, HyperLinkSpan, HyperText, HyperTextConfig,
    HyperTextHoverState, SpawnHyperTextExt, TopicContainer,
};
pub(crate) use hypertext::{
    apply_initial_visited_colors, apply_topic_lock_dimming, handle_topic_container, has_hypertext,
    hypertext_click, hypertext_hover, update_visited_link_colors,
};
pub(crate) use modal::{
    handle_dismiss_event, handle_modal_dismiss, process_modal_queue, reveal_modal_panel,
};
pub(crate) use progress_bar::update_progress_bars;
pub(crate) use scroll_view::{
    clamp_scroll_bounds, handle_scroll_input, handle_scrollbar_drag, handle_track_click,
    has_scroll_views, update_scrollbar_thumb,
};
pub(crate) use tabs::{handle_tab_clicks, sync_active_tab_marker, sync_tab_content_visibility};
pub(crate) use tooltip::{
    hide_tooltip, should_hide_tooltip, should_show_tooltip, show_tooltip, update_tooltip_hover,
};
#[cfg(feature = "viewport3d")]
pub(crate) use viewport3d::{
    has_viewports, viewport3d_cleanup, viewport3d_drag_rotate, viewport3d_track, Viewport3dTracked,
};
#[cfg(feature = "viewport3d")]
pub use viewport3d::{
    SpawnViewport3dExt, Viewport3d, Viewport3dCamera, Viewport3dConfig, Viewport3dDragState,
    Viewport3dHandle, Viewport3dPivot, Viewport3dRotation,
};
pub(crate) use visual::{update_border_visuals, update_interactive_visuals};
