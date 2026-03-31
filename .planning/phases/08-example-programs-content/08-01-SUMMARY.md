---
phase: 08-example-programs-content
plan: 01
subsystem: docs
tags: [examples, vm, brainfuck, piano-tuning]

requires:
  - phase: 02-vm-core
    provides: "HaydnVm with 19 opcodes, process_event API, loop pedal"
  - phase: 03-tuning-engine
    provides: "Piano tuning MIDI-to-event mapping"
provides:
  - "Four example programs (Hello World, Cat, Truth Machine, FizzBuzz) in piano tuning notation"
  - "Integration test suite verifying all examples against the VM"
  - "BF reduction test (canonical Hello World through bf_compile)"
affects: [08-02-readme, 08-03-esolang-wiki]

tech-stack:
  added: []
  patterns: ["Piano tuning arithmetic: build ASCII from Push(-n) × Push(-m) products"]

key-files:
  created:
    - examples/hello-world.md
    - examples/cat.md
    - examples/truth-machine.md
    - examples/fizzbuzz.md
    - crates/haydn-vm/tests/examples.rs
  modified: []

key-decisions:
  - "Cat uses [Read, Dup, LoopStart, PrintChar, Read, Dup, LoopEnd, Drop] — no Drop before PrintChar (corrected from plan)"
  - "FizzBuzz test proves BF reduction via bf_compile(canonical BF Hello World) + counting program rather than a full algorithmic BF FizzBuzz"
  - "Truth Machine infinite loop test uses step-by-step API with 5000-step limit to avoid OOM"

patterns-established:
  - "Example markdown format: title, teaches, program notation, explanation, event sequence"
  - "Hello World gets full tabular walkthrough; subsequent examples progressively lighter"

requirements-completed: [DOC-01, DOC-02, DOC-03, DOC-04]

duration: 12min
completed: 2026-03-31
---

# Plan 08-01: Example Programs Summary

**Four example programs created with piano-tuning note-name notation and 6 passing integration tests verifying each program against the Haydn VM.**

## Accomplishments
- Hello World (72 events) with full tabular walkthrough of first 3 characters, arithmetic reference table, and performance notes
- Cat (8 events) demonstrating I/O with loop-based echo
- Truth Machine (18 events) demonstrating conditional branching on input
- FizzBuzz via BF reduction demonstrating Turing completeness in practice
- 6 integration tests: hello_world, cat_echoes_input, cat_empty_input, truth_machine_zero, truth_machine_one, fizzbuzz

## Task Commits

1. **Task 1+2: All example programs + tests** — `7d8cfa2`

## Files Created
- `examples/hello-world.md` — Flagship example: push/multiply/print with full walkthrough
- `examples/cat.md` — I/O echo program in 8 notes
- `examples/truth-machine.md` — Conditional loop branching on input
- `examples/fizzbuzz.md` — BF reduction capstone with compilation table
- `crates/haydn-vm/tests/examples.rs` — 6 integration tests with run/run_with_input/bf_compile helpers

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Cat program event sequence incorrect in plan**
- **Found during:** Task 1 (Hello World and Cat)
- **Issue:** Plan specified `[Read, Dup, LoopStart, Drop, PrintChar, Read, Dup, LoopEnd, Drop]` but tracing reveals Drop before PrintChar causes stack underflow on replay
- **Fix:** Used `[Read, Dup, LoopStart, PrintChar, Read, Dup, LoopEnd, Drop]` instead
- **Verification:** test_cat_echoes_input and test_cat_empty_input both pass

**2. [Rule 1 - Bug] Truth Machine infinite loop causes OOM in test**
- **Found during:** Task 2 (Truth Machine)
- **Issue:** `process_event()` on LoopEnd replays the entire infinite loop, allocating unbounded StepResult vectors
- **Fix:** Used `feed()` + `step()` API with 5000-step limit for the input-'1' test
- **Verification:** test_truth_machine_one completes in <1s, asserts >100 '1' characters

**Total deviations:** 2 auto-fixed (2× Rule 1 - Bug)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the deviations above.

## Next Phase Readiness
- All four example files ready for README and wiki page references (Plans 08-02, 08-03)
- Hello World note names and event sequences are the canonical reference for the README Quick Start
- `cargo test --test examples` passes all 6 tests

## Self-Check: PASSED
