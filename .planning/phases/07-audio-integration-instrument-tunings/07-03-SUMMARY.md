---
phase: 07-audio-integration-instrument-tunings
plan: 03
subsystem: haydn-core
tags: [audio-capture, cli, dual-input, signal-level, reconnect]

requires:
  - phase: 07-01
    provides: Voice/guitar tunings + AudioSection accessor
  - phase: 07-02
    provides: AudioMsg::SignalLevel, audio_config_from_section, audio-aware TUI
provides:
  - --input mic CLI flag for audio capture mode
  - --audio-device flag for device selection
  - --list-audio flag for device enumeration
  - Dual-input event loop (MIDI or audio)
  - Audio disconnect/reconnect with VM state preservation
  - Signal level emission from analysis thread
affects: []

tech-stack:
  added: [cpal (direct dep for DeviceTrait)]
  patterns:
    - "Dual input via if/else branching (no premature abstraction)"
    - "Audio reconnect: drop old stream, recreate pipeline, reassign rx"

key-files:
  created: []
  modified:
    - crates/haydn/src/main.rs
    - crates/haydn/Cargo.toml
    - crates/haydn-audio/src/capture.rs

key-decisions:
  - "Code duplication between MIDI and audio event loops is preferred over premature abstraction"
  - "confidence → velocity mapping: (confidence * 127) as u8 for unified process_note interface"
  - "cpal added as direct dep of haydn for DeviceTrait::name()"

patterns-established:
  - "Signal level emitted as RMS of hop buffer per analysis cycle (~90 msgs/sec)"

requirements-completed: [AUD-05]

duration: 8min
completed: 2026-03-30
---

# Plan 07-03: Audio Capture Pipeline → Main Loop Integration Summary

**End-to-end audio path complete: mic → pitch detection → tuning engine → VM → TUI with dual-input CLI and disconnect/reconnect.**

## Performance

- **Tasks:** 2/2 complete
- **Files modified:** 3

## Accomplishments
- Added --input mic/midi, --audio-device, --list-audio CLI flags
- Audio mode: find device → construct AudioConfig from tuning → start_audio_capture → event loop
- Dual event loops for quiet and TUI modes (MIDI and audio branches)
- Audio TUI: confidence % rendering, signal level meter, "Mic:" status prefix
- Audio disconnect detection and reconnect with pipeline recreation + VM state preserved
- Signal level emission: RMS computed per hop in analysis thread, sent as AudioMsg::SignalLevel

## Task Commits

1. **Task 1+2: CLI flags + audio event loops + signal level** - `3a656d5` (feat)

## Deviations from Plan
- Added `cpal = "0.15"` direct dependency to haydn Cargo.toml for DeviceTrait::name() — plan's interfaces assumed device.name() would work without it.

## Issues Encountered
None.
