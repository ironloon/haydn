# Phase 3: Tuning System - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 03-tuning-system
**Areas discussed:** Mapping model, File structure, Validation & errors, Piano tuning design

---

## Mapping Model

### Q1: How should musical input map to VM events?

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed note mapping | Every note maps to exactly one event. Mode-switch note toggles between value/op mode. | |
| Interval-based | Two consecutive notes form a gesture; interval determines the event. | |
| Zone-based | Keyboard divided into zones (left hand = Push, right hand = Op). | |
| Layer/mode switching | Named layers with trigger note/CC to switch active layer. | |

**User's choice:** Free text — "previous phase discussions established that tuning file could do anything - does this mean all of this? and more?"
**Notes:** User correctly identified that Phase 1 decisions (D-04/D-05) made tuning files responsible for ALL mapping logic. The question shifted from "which mapping model" to "what primitives does the format support."

### Q2: How expressive should the tuning file format be?

| Option | Description | Selected |
|--------|-------------|----------|
| Rule-based engine | Each rule has trigger condition + action. Evaluated in order, first match wins. | |
| Fixed mapping types | Ship a small set of mapping types (note→event, interval→event, zone). | |
| Scripted/DSL | Embedded scripts (Lua/DSL) for maximum flexibility. | |

**User's choice:** Free text — "rules based engine sounds like what I might want, but I fear having regrets"
**Notes:** Concern about over/under-engineering. Led to follow-up with scoped starting point.

### Q3: Given that, which starting point?

| Option | Description | Selected |
|--------|-------------|----------|
| Rule engine with minimal triggers | 3 trigger types: exact note, note range, interval. Expandable later. | ✓ |
| Fixed types (note + interval) | 2 mapping types. Simpler now, harder to extend. | |
| Rule engine with all triggers | Full trigger set from day one. More upfront work. | |

**User's choice:** Rule engine with minimal triggers
**Notes:** Good balance — covers piano tuning needs, interval mappings, and is expandable.

### Q4: Should the default piano tuning use a specific mapping strategy?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, single default mode | Zones: left half pushes, right half triggers ops. | |
| Yes, with mode switching | Trigger note switches between push-mode and op-mode. | |
| Functional coverage only | Cover all 19 opcodes + value range. Don't optimize UX yet. | |

**User's choice:** Free text — "give me pros/cons/elaborate"
**Notes:** Extended discussion about the relationship between the tuning system (engine) and default tuning (one file). Clarified that engine supports everything, default tuning is a reference implementation.

### Q5: Confirmed: simple default tuning, expressive engine?

| Option | Description | Selected |
|--------|-------------|----------|
| Zone default, full engine | Zone-based default, rule engine supports anything. | ✓ |
| Defer piano layout | Skip piano design for now. | |

**User's choice:** Zone default, full engine

---

## File Structure

### Q1: How should a tuning TOML file be organized?

| Option | Description | Selected |
|--------|-------------|----------|
| Flat: metadata + rules array | One [metadata] + one [[rules]] array. Minimal, flat. | |
| Sectioned by concept | Separate sections for [values], [operations], [zones]. | |
| Layer-native schema | Nested [layers.*] sections with [switching]. | |

**User's choice:** Free text — "break it down more for me"
**Notes:** Created `.planning/scratch/tuning-schema-examples.md` with full examples of all three approaches for offline review.

### Q2: Which schema approach after seeing examples?

| Option | Description | Selected |
|--------|-------------|----------|
| A: Rules + shorthands | Shorthands for common patterns + [[rules]] for custom. | |
| B: Rules only | Only [[rules]]. Verbose but one concept. | |
| C: Sections only | Only predefined sections. Compact but limited. | |

**User's choice:** Free text — "what if the artist wants to define distance between notes as defining an opcode instead of a note itself?"
**Notes:** User raised interval-based mappings. Demonstrated how the rule engine handles intervals naturally as a trigger type. This confirmed the rules + shorthands approach.

### Q3: Does this confirm rules + shorthands?

| Option | Description | Selected |
|--------|-------------|----------|
| A: Rules + shorthands | Intervals, exact notes, and ranges are all trigger types. | ✓ |
| Revisit later | | |

**User's choice:** A: Rules + shorthands

### Q4: Rule evaluation order?

| Option | Description | Selected |
|--------|-------------|----------|
| Shorthands first, then rules | Shorthands are baseline, rules are overrides. | ✓ |
| Single merged list | Interleaved by source order. | |
| Agent's discretion | | |

**User's choice:** Shorthands first (after asking for recommendation)

---

## Validation & Errors

### Q1: Should a valid tuning file require ALL 19 opcodes?

| Option | Description | Selected |
|--------|-------------|----------|
| Require full opcode coverage | Must map all 19 or rejected. | |
| Allow partial with warnings | Warn on missing, don't reject. | |
| Agent's discretion | | |

**User's choice:** Free text — "allow partial with warnings, but maybe they can be suppressed by ack in the tuning file"
**Notes:** Led to `partial_coverage = true` metadata flag design.

### Q2: Warn + acknowledgment flag?

| Option | Description | Selected |
|--------|-------------|----------|
| Warn + acknowledgment flag | `partial_coverage = true` suppresses missing-opcode warnings. | ✓ |
| Always warn, no suppression | | |
| Agent's discretion | | |

**User's choice:** Warn + acknowledgment flag

### Q3: Error reporting style?

| Option | Description | Selected |
|--------|-------------|----------|
| First error only | Stop at first error. | |
| Collect all errors | Show all issues in one pass. | ✓ |
| Agent's discretion | | |

**User's choice:** Collect all + helpful messages (after asking for recommendation)

### Q4: Overlapping rules handling?

| Option | Description | Selected |
|--------|-------------|----------|
| Error on overlap | Two rules matching same note = error. | ✓ |
| Warn on overlap, first match wins | | |

**User's choice:** Error on overlap
**Notes:** Extended discussion about profiles, hot-swapping, and commenting alternatives. Conclusion: overlap = error within a file, profiles = separate files, alternatives = TOML comments, hot-swap = runtime feature for later phases.

---

## Piano Tuning Design

### Q1: Root note?

| Option | Description | Selected |
|--------|-------------|----------|
| Root = middle C (60) | Standard piano reference. | ✓ |
| Root = C3 (48) | Wider value range. | |
| Agent's discretion | | |

**User's choice:** Root = middle C (60)

### Q2: Value zone size?

| Option | Description | Selected |
|--------|-------------|----------|
| 2 octaves below middle C (24 notes) | MIDI 36-59, values -24 to -1. | ✓ |
| 3 octaves below (36 notes) | MIDI 24-59, larger range. | |
| Agent's discretion | | |

**User's choice:** 2 octaves below middle C

### Q3: Opcode layout?

| Option | Description | Selected |
|--------|-------------|----------|
| Chromatic ascending | C4=dup, C#4=swap... through all 19. | |
| Grouped by function | Stack ops on white keys, arithmetic on black. | |
| Agent's discretion | | |

**User's choice:** Free text — "maybe the default should be in a key of C or something"
**Notes:** User wants programming to sound *musical*. Led to the "musical from day one" decision.

### Q4: How much should default tuning prioritize musicality?

| Option | Description | Selected |
|--------|-------------|----------|
| Key of C (white keys = ops) | Diatonic C major. | |
| Chromatic now, musical later | Works for testing, refine Phase 8. | |
| Musical from day one | Design opcode layout for pleasant melodic sequences. | ✓ |

**User's choice:** Musical from day one

### Q5: Opcode assignment approach?

| Option | Description | Selected |
|--------|-------------|----------|
| Agent designs, you review | Agent analyzes common sequences, assigns for musicality. | ✓ |
| You design the layout | User designs based on musical intuition. | |
| Consonance-ranked assignment | Most-used opcodes get most consonant intervals. | |

**User's choice:** Agent designs, you review

### Q6: Black keys in operation zone?

| Option | Description | Selected |
|--------|-------------|----------|
| Common constants | Push 0, 1, -1, 10 etc. | |
| Unmapped | Safe but silent. | |
| Opcode aliases | | |
| Agent's discretion | ✓ | |

**User's choice:** Agent's discretion

---

## Agent's Discretion

- Crate structure (tuning within haydn-vm vs. separate crate)
- Specific white-key → opcode assignments for musicality
- Black key assignments in operation zone
- Internal rule engine data structures and matching algorithm
- TOML parsing library choice
- Test organization

## Deferred Ideas

- **Simulated performer tool** — separate crate/CLI that plays note sequences through computer audio + feeds VM. Placement: Phase 3.1.
- **Hot-swap tuning files at runtime** — switch tuning mid-performance. Phase 4/9.
- **Reserve `haydn-tuning` crate name** — consider during this phase.
- **Additional trigger types** (velocity, sequence, named zones) — expand rule engine as needed.
- **MIDI file export for preview** — alternative to real-time sim for hearing tuning musicality.
