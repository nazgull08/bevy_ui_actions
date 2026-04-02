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
    UiInputScope, UiTextExt, UiTheme,
};

// Re-export interactions
pub use interactions::{
    DragGhost, DragGhostStyle, DragPhase, DragState, Draggable, DropTarget, OnClick, OnDragCancel,
    OnDragStart, OnDrop, OnHover, OnHoverExit, OnPress, OnRightClick, PreviousInteraction,
};

// Re-export widgets
pub use widgets::{
    Active, BorderStyle, DialogueBox, DialogueConfig, DialogueContent, DialoguePosition,
    DialogueQueue, DialogueRequest, DialogueScroll, DialogueStyle, DialogueTopicButton,
    DialogueTopicPanel, Disabled, DismissDialogue, DismissDialogueEvent, DismissModal,
    DismissModalEvent, HyperLinkClicked, HyperLinkSpan, HyperText, HyperTextConfig,
    SpawnHyperTextExt, InteractiveVisual, ListItem, ListItemSelected, ListView, ListViewConfig,
    ListViewItems, Modal, ModalBackdrop, ModalPanel, ModalQueue, ModalRequest, ModalStyle,
    PanelConfig, ProgressBar, ProgressBarConfig, ProgressBarFill, ScrollDirection, ScrollView,
    ScrollViewConfig, Selected, SelectionMode, SpawnListViewExt, SpawnPanelExt,
    SpawnProgressBarExt, SpawnScrollViewExt, StatDiff, Tab, TabContent, TabGroup, Tooltip,
    TooltipBuilder, TooltipContent, TooltipSection, TooltipSet, TooltipState, TooltipStyle,
    TooltipUI, TopicDiscovered, TopicEntry, TopicRegistry, VisualStyle, append_dialogue_text,
    spawn_modal_button,
};
