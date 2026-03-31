---
plan: 06-03
phase: 06-audio-capture-pitch-detection
status: complete
started: 2026-03-30
completed: 2026-03-30
---

## Summary

Implemented onset detection, noise gating, and the note state machine that converts frame-by-frame analysis results into discrete NoteOn/NoteOff events.

## What Was Built

- **NoiseGate**: RMS-based silence/signal classification with configurable dB threshold. `rms_db()` utility and `is_active()` gate check.
- **OnsetDetector**: Energy spike detection using exponential moving average baseline. Detects sudden loudness increases without triggering on sustained signal.
- **NoteStateMachine**: State machine (SILENCE→ONSET→ACTIVE) producing AudioMsg events. Handles onset confirmation via pitch, debounce via min_note_ms, note changes, and sustained notes without duplicate events.

## Key Files

### Created
- `crates/haydn-audio/src/gate.rs` — NoiseGate with RMS dB computation
- `crates/haydn-audio/src/onset.rs` — OnsetDetector with EMA baseline
- `crates/haydn-audio/src/state_machine.rs` — NoteStateMachine with full lifecycle
- `crates/haydn-audio/tests/onset_detection.rs` — Integration tests for state machine

### Modified
- `crates/haydn-audio/src/lib.rs` — Added module declarations and re-exports

## Test Results

- 31 unit tests passing (gate: 5, onset: 3, state machine: 5, pitch: 15, types: 3)
- 10 integration tests passing (onset_detection: 6, pitch_accuracy: 4)

## Deviations

None.
