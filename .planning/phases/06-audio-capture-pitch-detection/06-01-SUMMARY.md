---
plan: 06-01
phase: 06-audio-capture-pitch-detection
status: complete
started: 2026-03-30
completed: 2026-03-30
---

## Summary

Scaffolded the `haydn-audio` crate with core types and the PitchDetector trait, and extended `haydn-tuning` with an optional `[audio]` configuration section.

## What Was Built

- **haydn-audio crate**: New workspace member with `AudioMsg` (NoteOn/NoteOff), `AudioConfig` (9 parameters with defaults), `AudioError` (3 variants), `PitchDetector` trait, `PitchEstimate` struct, and `freq_to_midi()` helper
- **Tuning [audio] section**: `AudioSection` type in haydn-tuning with 6 configurable parameters, serde defaults, validation for all ranges, backward-compatible (Option field)
- **Piano defaults**: `[audio]` section added to piano.toml with default values
- **Documentation**: TUNING.md updated with `[audio]` reference table, default example, and voice-optimized example

## Key Files

### Created
- `crates/haydn-audio/Cargo.toml` — Crate manifest (cpal, ringbuf, thiserror deps)
- `crates/haydn-audio/src/lib.rs` — Crate root with module declarations and re-exports
- `crates/haydn-audio/src/types.rs` — AudioMsg, AudioConfig, AudioError
- `crates/haydn-audio/src/pitch/mod.rs` — PitchDetector trait, PitchEstimate, freq_to_midi

### Modified
- `Cargo.toml` — Added haydn-audio to workspace members
- `crates/haydn-tuning/src/types.rs` — Added AudioSection struct, added audio field to RawTuningFile
- `crates/haydn-tuning/src/validate.rs` — Added validate_audio() for [audio] section validation
- `crates/haydn-tuning/tunings/piano.toml` — Added [audio] section with defaults
- `docs/TUNING.md` — Added [audio] documentation section

## Test Results

- 8 haydn-audio unit tests passing (AudioMsg, AudioConfig defaults, freq_to_midi accuracy)
- 29 existing haydn-tuning tests passing (backward compatibility confirmed)

## Deviations

None.
