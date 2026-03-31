---
phase: 01-language-specification
plan: 01
subsystem: documentation
tags: [specification, vm, stack-machine, esoteric-language]

requires: []
provides:
  - "Formal language spec sections 1-6: Introduction, Computational Model, Instruction Set, Event Interface, Control Flow, Silence & Timing"
  - "19-opcode instruction set with complete stack effects and edge case behaviors"
  - "Loop pedal state machine with full transition table"
  - "Event interface contract (Push/Op) for tuning system boundary"
affects: [01-02, phase-02-vm-core, phase-03-tuning-system]

tech-stack:
  added: []
  patterns:
    - "Forth-notation stack effects for opcode documentation"
    - "State machine transition table for control flow semantics"

key-files:
  created:
    - docs/SPECIFICATION.md
  modified: []

key-decisions:
  - "Signed i64 values with wrapping arithmetic — supports negative semitone offsets"
  - "Edge case philosophy: VM never panics, never halts — all edge cases have defined behavior"
  - "loop_start peeks (does not consume), loop_end pops (consumes) — enables while-loop semantics"
  - "Live input queued during replay — prevents chaotic interleaving"
  - "Max loop nesting depth: 256"
  - "Push is not an opcode — distinct event type"
  - "v1 VM ignores event metadata (velocity, duration) — forward compatibility hook"

patterns-established:
  - "Specification structure: numbered sections 1-N with subsections"
  - "Opcode table format: Opcode | Stack Effect | Description | Edge Case Behavior"
  - "State machine transition table: Current State | Event | Action | Next State"

requirements-completed: [SPEC-01, SPEC-02, SPEC-04, SPEC-05, SPEC-06, SPEC-07, SPEC-08]

duration: 5min
completed: 2026-03-28
---

# Plan 01-01: Core Language Specification Summary

**Complete Haydn VM specification (sections 1–6) covering the computational model, all 19 opcodes with edge cases, the Push/Op event interface, a full loop state machine transition table, and silence/timing semantics.**

## Performance

- **Duration:** ~5 min
- **Tasks:** 2 completed (both written as one cohesive document)
- **Files created:** 1

## Accomplishments
- Wrote the foundational specification document that all subsequent implementation phases depend on
- Defined all 19 opcodes with Forth-notation stack effects, descriptions, and deterministic edge case behaviors
- Created complete loop pedal state machine with 15-row transition table covering all state/event combinations
- Established the Event interface contract (Push/Op) as the sole boundary between tuning system and VM
- Documented while-loop semantics via peek/pop distinction (loop_start peeks, loop_end pops)

## Task Commits

1. **Task 1 + Task 2: Write complete specification (sections 1–6)** - `9bb7407` (docs)

## Files Created/Modified
- `docs/SPECIFICATION.md` - Complete Haydn language specification: Introduction, Computational Model, Instruction Set, Event Interface, Control Flow (Loop Pedal), Silence & Timing

## Decisions Made
- Wrote all 6 sections as a single cohesive document rather than splitting across two separate commits, since the event interface (§4) and control flow (§5) reference the instruction set (§3) throughout

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cohesive document structure**
- **Found during:** Task 1 (VM core specification)
- **Issue:** Tasks 1 and 2 target the same file with heavy cross-references between sections
- **Fix:** Wrote all 6 sections together for consistency, verified both task acceptance criteria independently
- **Verification:** All acceptance criteria for both tasks pass
- **Committed in:** 9bb7407

---

**Total deviations:** 1 auto-fixed (Rule 3: Blocking)
**Impact on plan:** Positive — single cohesive document rather than potentially inconsistent append. All criteria verified independently.

## Issues Encountered
None

## Self-Check: PASSED
- [x] `docs/SPECIFICATION.md` exists on disk
- [x] `git log --oneline --grep="01-01"` returns ≥1 commit
- [x] All 19 opcodes present in instruction set table
- [x] Event interface, loop state machine, silence/timing sections complete
- [x] All acceptance criteria verified for both tasks
