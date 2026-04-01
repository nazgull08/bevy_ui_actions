pub mod core;
pub mod interactions;
pub mod widgets;

mod plugin;
pub mod prelude;

// Re-export plugin
pub use plugin::UiActionsPlugin;

// Re-export core
pub use core::{
    ButtonConfig, ButtonStyle, NodeExt, SpawnActionButton, SpawnUiExt, TextRole, UiAction,
    UiTextExt, UiTheme,
};

// Re-export interactions
pub use interactions::{
    DragGhost, DragGhostStyle, DragPhase, DragState, Draggable, DropTarget, OnClick, OnDragCancel,
    OnDragStart, OnDrop, OnHover, OnHoverExit, OnPress, OnRightClick, PreviousInteraction,
};

// Re-export widgets
pub use widgets::{
    Active, BorderStyle, Disabled, InteractiveVisual, PanelConfig, ProgressBar, ProgressBarConfig,
    ProgressBarFill, ScrollDirection, ScrollView, ScrollViewConfig, Selected, SpawnPanelExt,
    SpawnProgressBarExt, SpawnScrollViewExt, StatDiff, Tab, TabContent, TabGroup, Tooltip,
    TooltipBuilder, TooltipContent, TooltipSection, TooltipSet, TooltipState, TooltipStyle,
    TooltipUI, VisualStyle,
};
