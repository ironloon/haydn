---
phase: 03-tuning-system
plan: 02
subsystem: tuning
tags: [engine, piano, midi, integration-tests, vm-roundtrip]

requires:
  - phase: 03-01
    provides: "RawTuningFile types, parse_opcode(), validate(), TuningError"
provides:
  - "TuningEngine with compile(), map_note(), reset()"
  - "Default piano tuning (19 opcodes, white keys C4-G6, value zone 36-59)"
  - "load_tuning(), load_tuning_file(), default_piano_tuning() public API"
affects: [03.1-simulated-performer, 04-midi-input]

tech-stack:
  added: []
  patterns: [compiled-rule-matching, interval-tracking, include_str-embedding]

key-files:
  created:
    - crates/haydn-tuning/src/engine.rs
    - crates/haydn-tuning/tunings/piano.toml
    - crates/haydn-tuning/tests/integration.rs
  modified:
    - crates/haydn-tuning/src/lib.rs
    - crates/haydn-vm/src/types.rs

key-decisions:
  - "Added Hash derive to haydn_vm::Opcode to enable HashSet-based coverage testing"
  - "Overlap validation rejects overlapping exact-note and value-range rules at compile time"
  - "last_note always updated even on no-match for consistent interval tracking"

patterns-established:
  - "First-match-wins rule evaluation: values → operations → raw rules"
  - "include_str! embedding for built-in tuning files"

requirements-completed: [TUNE-02, TUNE-03, TUNE-06]

duration: 8min
completed: 2026-03-29
---

# Phase 03 Plan 02: TuningEngine and Piano Tuning Summary

**Rule-matching engine that compiles TOML tuning files into fast note→Event resolvers, with default piano tuning embedding all 19 opcodes on white keys**

## Performance

- **Tasks:** 2
- **Files created:** 3
- **Files modified:** 2
- **Tests:** 17 integration tests passing

## Accomplishments

1. TuningEngine::compile() validates raw tuning and builds ordered rule list
2. map_note() resolves MIDI notes to Events with first-match-wins semantics
3. Interval tracking across consecutive notes for pattern-based mappings
4. Default piano tuning: values zone MIDI 36-59, 19 opcodes on white keys C4-G6
5. Public API: load_tuning(), load_tuning_file(), default_piano_tuning()
6. 17 integration tests: piano mappings, custom tunings, intervals, VM round-trips
7. Verified full workspace (131 tests) with zero regressions

## Deviations from Plan

- **[Rule 1 - Bug] Hash derive:** Added Hash to haydn_vm::Opcode to support HashSet in test. Safe, backward-compatible change.
- **[Rule 1 - Bug] First-match test:** Original test had overlapping value zone and raw rule for same note — validator correctly rejects overlaps. Fixed test to use non-overlapping notes.

## Issues Encountered

None

## Next Plan Readiness

Ready for Plan 03-03 (TUNING.md documentation).
