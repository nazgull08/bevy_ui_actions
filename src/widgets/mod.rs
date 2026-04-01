mod list_view;
mod modal;
mod panel;
mod progress_bar;
mod scroll_view;
mod tabs;
mod tooltip;
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
pub(crate) use visual::{update_border_visuals, update_interactive_visuals};
