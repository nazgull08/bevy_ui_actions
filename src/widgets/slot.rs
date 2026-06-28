//! Slot grid widget — a grid of drop-target slots bound to a container entity.
//!
//! Generic over the container: a `Slot` knows its owning `Entity` and index, so a
//! single component + a single `dragged_slot` helper replace the per-container
//! marker types and "search both queries" boilerplate of bespoke inventories.
//!
//! Following the library convention (see `list_view`): the widget provides the
//! grid structure and per-slot behavior wiring, while the caller supplies each
//! slot's **content** via a closure and owns the data→visual sync.

use crate::core::UiAction;
use crate::interactions::{DragState, DropTarget, OnDrop};
use bevy::prelude::*;

/// A slot bound to a container entity and its index within that container.
///
/// `container` is opaque to the widget — it is whatever entity holds the data
/// (an inventory component, a chest, …). Drop actions and sync systems use it to
/// resolve which container/index a slot refers to.
#[derive(Component, Clone, Copy, Debug)]
pub struct Slot {
    pub container: Entity,
    pub index: usize,
}

/// Visual configuration for a [`spawn_slot_grid`](SpawnSlotGridExt::spawn_slot_grid).
#[derive(Clone, Debug)]
pub struct SlotGridConfig {
    pub slot_size: f32,
    pub gap: f32,
    pub columns: usize,
    /// Base background (the caller's sync typically recolors filled slots).
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
}

impl Default for SlotGridConfig {
    fn default() -> Self {
        Self {
            slot_size: 60.0,
            gap: 6.0,
            columns: 4,
            background: Color::srgb(0.18, 0.18, 0.20),
            border_color: Color::srgb(0.35, 0.35, 0.40),
            border_width: 2.0,
        }
    }
}

/// Extension trait for spawning a slot grid.
pub trait SpawnSlotGridExt {
    /// Spawn a wrapping grid of `count` slots bound to `container`.
    ///
    /// Each slot gets `Slot { container, index }`, `DropTarget`,
    /// `OnDrop::new(make_drop(index))`, `Interaction`, and a base node. `content`
    /// builds each slot's children (icon, name, …) — empty for a bare grid.
    /// `Draggable` is **not** added (it depends on content; the caller toggles it).
    fn spawn_slot_grid<A, MakeDrop, Content>(
        &mut self,
        config: &SlotGridConfig,
        container: Entity,
        count: usize,
        make_drop: MakeDrop,
        content: Content,
    ) -> EntityCommands<'_>
    where
        A: UiAction,
        MakeDrop: Fn(usize) -> A,
        Content: Fn(&mut ChildSpawnerCommands, usize);
}

impl SpawnSlotGridExt for ChildSpawnerCommands<'_> {
    fn spawn_slot_grid<A, MakeDrop, Content>(
        &mut self,
        config: &SlotGridConfig,
        container: Entity,
        count: usize,
        make_drop: MakeDrop,
        content: Content,
    ) -> EntityCommands<'_>
    where
        A: UiAction,
        MakeDrop: Fn(usize) -> A,
        Content: Fn(&mut ChildSpawnerCommands, usize),
    {
        let columns = config.columns.max(1) as f32;
        let row_width = columns * config.slot_size + (columns - 1.0) * config.gap;

        let slot_size = config.slot_size;
        let border_width = config.border_width;
        let background = config.background;
        let border_color = config.border_color;

        let mut grid = self.spawn(Node {
            width: Val::Px(row_width),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(config.gap),
            column_gap: Val::Px(config.gap),
            ..default()
        });

        grid.with_children(|grid| {
            for index in 0..count {
                grid.spawn((
                    Node {
                        width: Val::Px(slot_size),
                        height: Val::Px(slot_size),
                        border: UiRect::all(Val::Px(border_width)),
                        ..default()
                    },
                    BackgroundColor(background),
                    BorderColor(border_color),
                    Slot { container, index },
                    DropTarget,
                    OnDrop::new(make_drop(index)),
                    Interaction::None,
                ))
                .with_children(|slot| content(slot, index));
            }
        });

        grid
    }
}

/// The slot currently being dragged, if any (resolved from [`DragState`]).
///
/// Replaces the "iterate every slot query to find the drag source" boilerplate.
pub fn dragged_slot(world: &World) -> Option<Slot> {
    world
        .resource::<DragState>()
        .dragging
        .and_then(|entity| world.get::<Slot>(entity).copied())
}
