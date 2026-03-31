---
phase: 07-audio-integration-instrument-tunings
plan: 02
subsystem: haydn-core
tags: [audio-config, tui, display, signal-level]

requires:
  - phase: 07-01
    provides: AudioSection accessor on TuningEngine
provides:
  - AudioMsg::SignalLevel(f32) variant for signal level data
  - haydn-audio dependency in haydn crate
  - audio_config_from_section() conversion helper
  - Audio-aware TUI display model (confidence, input_mode, signal_level)
affects: [07-03]

tech-stack:
  added: [haydn-audio dependency]
  patterns:
    - "Dual-mode rendering: MIDI shows velocity, audio shows confidence %"
    - "Signal meter using Unicode block characters"

key-files:
  created: []
  modified:
    - crates/haydn-audio/src/types.rs
    - crates/haydn/Cargo.toml
    - crates/haydn/src/lib.rs
    - crates/haydn/src/display.rs
    - crates/haydn/src/main.rs

key-decisions:
  - "confidence field is Option<f32> — None for MIDI, Some for audio — preserves backward compat"
  - "TuiState::new() takes input_mode parameter; existing MIDI callers pass 'MIDI'"

patterns-established:
  - "audio_config_from_section bridges tuning and audio subsystems"

requirements-completed: [AUD-05]

duration: 5min
completed: 2026-03-30
---

# Plan 07-02: Audio Dependency + TUI Display Model Summary

**AudioMsg extended with SignalLevel, haydn-audio wired as dependency, TUI renders confidence % for audio and velocity for MIDI.**

## Performance

- **Tasks:** 2/2 complete
- **Files modified:** 5

## Accomplishments
- Added AudioMsg::SignalLevel(f32) variant for signal level emission
- Added haydn-audio as path dependency of haydn crate
- Created audio_config_from_section() to bridge AudioSection → AudioConfig
- Extended HistoryEntry with optional confidence field
- Extended TuiState with input_mode and signal_level fields
- Operations panel conditionally renders "[C4 v=80]" or "[C4 ~92%]"
- Status bar shows input mode prefix and Unicode signal meter in mic mode

## Task Commits

1. **Task 1+2: All Plan 07-02 changes** - `26f2363` (feat)

## Deviations from Plan
None — plan executed exactly as written.

## Issues Encountered
None.
