---
phase: 07-audio-integration-instrument-tunings
plan: 01
subsystem: tuning
tags: [toml, voice, guitar, audio-section]

requires: []
provides:
  - Default voice tuning (voice.toml) with dense chromatic mapping
  - Default guitar tuning (guitar.toml) with dense chromatic mapping
  - Built-in loaders default_voice_tuning() and default_guitar_tuning()
  - AudioSection accessor on TuningEngine
affects: [07-02, 07-03]

tech-stack:
  added: []
  patterns:
    - "include_str! for built-in tuning files"
    - "Dense chromatic mapping: values below root, operations at/above root"

key-files:
  created:
    - crates/haydn-tuning/tunings/voice.toml
    - crates/haydn-tuning/tunings/guitar.toml
  modified:
    - crates/haydn-tuning/src/engine.rs
    - crates/haydn-tuning/src/lib.rs
    - crates/haydn-tuning/tests/integration.rs

key-decisions:
  - "Voice and guitar both use dense chromatic mapping (every semitone meaningful)"
  - "Voice tuning has generous thresholds for expressive singing; guitar is tighter for precise plucks"

patterns-established:
  - "audio field stored in TuningEngine and exposed via audio_section() accessor"

requirements-completed: [TUNE-04, TUNE-05]

duration: 5min
completed: 2026-03-30
---

# Plan 07-01: Voice/Guitar Tunings + AudioSection Accessor Summary

**Three built-in tunings (piano, voice, guitar) with AudioSection exposure for downstream audio pipeline configuration.**

## Performance

- **Tasks:** 2/2 complete
- **Files modified:** 5

## Accomplishments
- Created voice.toml with 10 value notes (A2-F#3) and 19 opcodes (G3-C#5), tuned for forgiving vocal input
- Created guitar.toml with 12 value notes (E2-D#3) and 19 opcodes (E3-Bb4), tuned for sharp transient detection
- Added `audio` field to TuningEngine + `audio_section()` accessor
- Added `default_voice_tuning()` and `default_guitar_tuning()` built-in loaders
- 18 new integration tests covering all tuning zones, opcodes, and audio sections

## Task Commits

1. **Task 1+2: Voice/guitar tunings + loaders + accessor** - `ee55744` (feat)

## Files Created/Modified
- `crates/haydn-tuning/tunings/voice.toml` - Voice tuning with rich inline comments
- `crates/haydn-tuning/tunings/guitar.toml` - Guitar tuning with instrument-specific audio params
- `crates/haydn-tuning/src/engine.rs` - AudioSection field + accessor on TuningEngine
- `crates/haydn-tuning/src/lib.rs` - include_str! + default_voice/guitar_tuning() loaders
- `crates/haydn-tuning/tests/integration.rs` - 18 new tests for voice, guitar, audio sections

## Deviations from Plan
None — plan executed exactly as written.

## Issues Encountered
None.
