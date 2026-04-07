#![cfg_attr(docsrs, feature(doc_cfg))]
//! # bevy_ui_actions
//!
//! Action-driven UI toolkit for [Bevy](https://bevyengine.org/) game engine.
//!
//! ## Core
//! - **[`UiAction`]** trait — exclusive `World` access on click/hover/press
//! - **Interactions** — [`OnClick`], [`OnRightClick`], [`OnHover`], [`OnHoverExit`], [`OnPress`]
//! - **Drag & Drop** — [`Draggable`], [`DropTarget`], [`OnDrop`], [`OnDragCancel`]
//! - **[`UiTheme`]** — font + sizes + colors by [`TextRole`]. [`UiTextExt`] sugar
//! - **[`NodeExt`]** — layout presets: `row()`, `column()`, `centered()`, `fill()`, `padded()`
//!
//! ## Widgets
//! - **[`TabGroup`]** / **[`Tab`]** / **[`TabContent`]** — tabbed UI
//! - **[`ProgressBar`]** — configurable progress bars
//! - **[`Tooltip`]** — rich tooltips (title/subtitle/stats/separator/text)
//! - **[`InteractiveVisual`]** — hover/press/selected/disabled color states
//! - **[`PanelConfig`]** — styled panels with presets (`dark`, `overlay`, `sidebar`)
//! - **[`ScrollViewConfig`]** — scrollable containers with scrollbar
//! - **[`ListViewConfig`]** — selectable item lists
//! - **[`ModalQueue`]** / **[`ModalRequest`]** — modal dialogs with backdrop + ESC dismiss
//! - **[`HyperText`]** — inline clickable links with glyph-level hit-testing
//! - **[`DialogueQueue`]** — Morrowind-style dialogue with topic registry + topic panel
//! - **[`TopicContainer`]** — generic topic expansion outside dialogue (works with [`HyperText`] links)
//! - **[`Viewport3d`]** — 3D render-to-texture preview *(feature `viewport3d`)*
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_ui_actions::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(UiActionsPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! struct MyAction;
//! impl UiAction for MyAction {
//!     fn execute(&self, world: &mut World) {
//!         println!("Clicked!");
//!     }
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2d);
//!     commands.spawn(Node::default()).with_children(|root| {
//!         root.spawn_button("Click me", MyAction);
//!     });
//! }
//! ```
//!
//! ## Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `viewport3d` | 3D render-to-texture viewport widget (enables `bevy_core_pipeline` + `bevy_pbr`) |

pub mod core;
pub mod interactions;
pub mod widgets;

mod plugin;
pub mod prelude;

// Re-export plugin
pub use plugin::UiActionsPlugin;

// Re-export core
pub use core::{
    ButtonConfig, ButtonStyle, NodeExt, SpawnActionButton, SpawnUiExt, TextPreset, TextRole,
    UiAction, UiInputScope, UiTextExt, UiTheme, UiThemedText,
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
    TooltipUI, TopicContainer, TopicDiscovered, TopicEntry, TopicRegistry, VisualStyle,
    append_dialogue_text, append_topic_block, spawn_modal_button,
};

#[cfg(feature = "viewport3d")]
pub use widgets::{
    Viewport3d, Viewport3dCamera, Viewport3dConfig, Viewport3dDragState, Viewport3dHandle,
    Viewport3dPivot, Viewport3dRotation, SpawnViewport3dExt,
};
