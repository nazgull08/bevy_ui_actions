# Changelog

All notable changes to `bevy_ui_actions` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/).

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
