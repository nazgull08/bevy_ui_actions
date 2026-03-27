mod progress_bar;
mod tabs;
mod tooltip;
mod visual;

pub use progress_bar::{ProgressBar, ProgressBarConfig, ProgressBarFill, SpawnProgressBarExt};
pub use tabs::{Tab, TabContent, TabGroup};
pub use tooltip::{
    StatDiff, Tooltip, TooltipBuilder, TooltipContent, TooltipSection, TooltipSet, TooltipState,
    TooltipStyle, TooltipUI,
};
pub use visual::{Active, BorderStyle, Disabled, InteractiveVisual, Selected, VisualStyle};

// Re-export systems for plugin
pub(crate) use progress_bar::update_progress_bars;
pub(crate) use tabs::{handle_tab_clicks, sync_active_tab_marker, sync_tab_content_visibility};
pub(crate) use tooltip::{
    hide_tooltip, should_hide_tooltip, should_show_tooltip, show_tooltip, update_tooltip_hover,
};
pub(crate) use visual::{update_border_visuals, update_interactive_visuals};
