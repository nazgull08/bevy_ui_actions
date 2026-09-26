# Changelog

## 0.3.1

- **Fix:** hypertext link hit-testing on HiDPI. Bevy 0.19 lays glyphs out in
  physical pixels (`glyph.position`, `atlas_info.rect`) while
  `TextLayoutInfo::size` is logical; 0.3.0 built link rects from raw glyph
  coordinates and compared them with a logical cursor, so on a 2.0 scale
  factor every link landed twice as far from the node origin and twice as
  large. Glyph coordinates are now scaled by the node's inverse scale factor.

All notable changes to `bevy_ui_actions` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/).

## [0.3.0]

Bevy 0.19 support (migrated across 0.17 → 0.18 → 0.19).

### Changed
- **Bevy 0.19.1** (from 0.16). All breaking engine changes absorbed
  internally; the widget API is unchanged except where Bevy types leak
  through configs (see below).
- Buffered events are Bevy **Messages** now: `SliderChanged`,
  `HyperLinkClicked`, dialogue events etc. are read with
  `MessageReader` on the consumer side.
- Hit-testing (hypertext links, slider, window drag, scroll thumb,
  viewport3d) reworked for 0.17+ UI transforms (`UiGlobalTransform`)
  and 0.19 Parley text (`PositionedGlyph` layout); logical-pixel space
  throughout — correct on HiDPI.
- `bevy_ui_render` feature is enabled by the library (0.17+ splits UI
  rendering out; without it consumers with `default-features = false`
  would render no UI at all).

### Fixed
- Hypertext link clicks/hover after the 0.17 UI-transform split (the
  old `GlobalTransform` query silently matched nothing).

### Note
- Temporary `encase = "=0.12.1"` version anchor: encase 0.12.2 moved to
  syn 3 in a patch release and breaks any fresh lockfile against current
  Bevy (still on syn 2). The anchor will be removed once Bevy ships a
  syn-3 build (bevyengine/bevy#25846).

## [0.2.7]

Slider widget for tuning panels.

### Added
- **`Slider` widget** — horizontal value slider: draggable thumb +
  click-anywhere-on-track jump, `min..=max` range with optional `step`
  snap, optional fill strip. Spawn via `SpawnSliderExt::spawn_slider`
  with `SliderConfig` (presets: `symmetric`, `degrees`).
- **`SliderChanged` event** — emitted only for mouse-driven changes;
  programmatic `Slider::set` moves the thumb without emitting, so
  loading presets never echoes back into the handler.
- **`slider` example** — RGB color mixer + a tuning-panel row composing
  a slider with −/+ fine-step buttons.

Respects `UiInputScope` and window scale factor (logical/physical px).

## [0.2.6]

Dialogue: a discoverable close affordance and a single close signal.

### Added
- **Close ("Goodbye") button** for the dialogue box. Enable via
  `DialogueConfig::show_close_button` (default `false`); label via
  `DialogueConfig::close_button_label` (default `"Goodbye"`). It sits at the
  bottom-right of the panel and uses the choice-button palette for hover/press.
- **`DialogueCloseRequested` event** — the unified "player asked to close"
  signal. The default handler dismisses on it, so a standalone dialogue closes
  with no extra wiring; games that own external state (input focus, pause) can
  listen to this one event instead and drive their own teardown.

### Changed
- **ESC now emits `DialogueCloseRequested`** (still gated by
  `close_on_esc`) instead of `DismissDialogueEvent` directly, so ESC and the
  close button share one code path. Behavior is unchanged for standalone use —
  the default sink still dismisses.
