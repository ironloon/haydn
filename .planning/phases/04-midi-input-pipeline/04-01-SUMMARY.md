---
phase: 04-midi-input-pipeline
plan: 01
subsystem: midi-pipeline
tags: [midir, midly, clap, mpsc, real-time]

requires:
  - phase: 02-vm-core
    provides: "HaydnVm with process_event(), stack, output, close"
  - phase: 03-tuning-system
    provides: "TuningEngine with map_note(), default_piano_tuning(), load_tuning_file()"
provides:
  - "Complete MIDI → tuning → VM → event log pipeline in haydn binary"
  - "CLI with --list-midi, --midi-device, --tuning flags"
  - "MIDI device enumeration and selection (auto, filter, interactive)"
  - "Named midi_callback function reusable for reconnect"
  - "Scrolling event log with note name, velocity, mapping, stack, output, edge cases"
affects: [04-02-disconnect-reconnect]

tech-stack:
  added: [midir 0.10, midly 0.5, clap 4, ctrlc 3, anyhow 1]
  patterns: [mpsc channel for MIDI callback isolation, recv_timeout event loop, named callback for reuse]

key-files:
  created: []
  modified:
    - crates/haydn/Cargo.toml
    - crates/haydn/src/main.rs

key-decisions:
  - "Combined Tasks 1+2 into single commit — they share the same file and Task 2 builds directly on Task 1's scaffolding"
  - "Used named function midi_callback instead of closure for reusability in Plan 02 reconnect"
  - "Session summary with vm.close() built into main loop exit path (ready for Plan 02 enhancement)"

patterns-established:
  - "MIDI callback pattern: named fn + mpsc::Sender as data parameter"
  - "Event log format: [NoteName v=vel] → Mapping  |  Stack: [...]  |  Out/Edge"

requirements-completed: [MIDI-01, MIDI-02, MIDI-03]

duration: 5min
completed: 2026-03-29
---

# Phase 04 Plan 01: MIDI Capture + Full Pipeline Summary

**End-to-end MIDI → tuning → VM → event log pipeline in the haydn binary, delivering the first playable experience**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-29T22:50:00Z
- **Completed:** 2026-03-29T22:55:00Z
- **Tasks:** 2 (combined into 1 atomic commit)
- **Files modified:** 2

## Accomplishments
- Complete MIDI input pipeline: midir captures note events, midly parses them, tuning engine maps to VM events, VM executes, event log displays results
- CLI with --list-midi (enumerate ports), --midi-device (substring filter), --tuning (custom TOML file)
- Device selection handles zero/one/multiple devices with clear error messages
- Event log shows note name, velocity, mapping result, stack state, VM output, and edge cases
- Ctrl+C handler with session summary (final stack + program output)

## Task Commits

1. **Task 1+2: CLI + device selection + MIDI pipeline + event log** - `9ece633` (feat)

**Plan metadata:** pending

## Files Created/Modified
- `crates/haydn/Cargo.toml` - Added dependencies: haydn-vm, haydn-tuning, midir, midly, clap, ctrlc, anyhow
- `crates/haydn/src/main.rs` - Complete MIDI pipeline: CLI, device selection, callback, tuning, VM loop, event log, session summary

## Deviations from Plan

**[Rule 3 - Blocking] Combined Task 1+2** — Tasks 1 and 2 target the same file (main.rs) with Task 2 building directly on Task 1's scaffolding. Splitting would have required artificial placeholder code. Combined into a single coherent implementation.

**Total deviations:** 1 auto-fixed (structural). **Impact:** None — all acceptance criteria from both tasks are met.

## Self-Check: PASSED
