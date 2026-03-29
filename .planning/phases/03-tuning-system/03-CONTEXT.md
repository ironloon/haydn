# Phase 3: Tuning System - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the configurable tuning file format, parser, validator, and default piano tuning. The tuning system is the bridge between musical input and the VM — it takes musical note events (MIDI note numbers) and produces `Event::Push(i64)` and `Event::Op(Opcode)` values that the VM consumes. This phase delivers the tuning engine as a library (likely within `haydn-vm` or a new `haydn-tuning` crate), the TOML schema, a validator, the default piano tuning file, and documentation for custom tuning authorship.

</domain>

<decisions>
## Implementation Decisions

### Mapping Model
- **D-01:** Rule-based engine with minimal trigger types. The tuning file is a rule engine — each rule has a trigger condition and an action (emit Push or Op). Rules evaluated top-to-bottom, first match wins.
- **D-02:** Three trigger types for v1: **exact note**, **note range**, and **interval** (semitone distance between consecutive notes). More trigger types (velocity, zone, sequence) can be added later without schema breaks.
- **D-03:** The rule engine is the universal mapping mechanism. Any mapping strategy (zones, modes, interval-based, alternating) is expressible as a set of rules. The engine imposes no preferred model — tuning authors compose rules to build whatever strategy they want.

### File Structure (TOML Schema)
- **D-04:** Rules + shorthands format. Two shorthand sections — `[values]` (note range → Push with semitone offset) and `[operations]` (note → Op table) — cover 90% of tuning files concisely. Raw `[[rules]]` available for anything custom or creative.
- **D-05:** Evaluation order: **shorthands first, then raw rules.** Shorthands are the base mapping; raw rules are overrides/additions. First match wins across the combined rule set.
- **D-06:** `[metadata]` section includes: `name`, `root_note` (MIDI number, semitone offsets calculated from this), `instrument` (string), `author` (optional), and `partial_coverage` (bool, see D-09).

### Validation & Errors
- **D-07:** Partial opcode coverage allowed with warnings. A tuning file does NOT need to map all 19 opcodes — it can be a specialized tuning (e.g., "arithmetic only"). Missing opcodes produce warnings by default.
- **D-08:** Collect all errors in one pass. Validation reports every issue found — syntax errors, invalid opcode names, overlapping rules, out-of-range note numbers — with the TOML key/section, what's wrong, and what was expected. Includes typo suggestions for opcode names (e.g., "dupe" → "did you mean 'dup'?").
- **D-09:** Acknowledgment flag: `partial_coverage = true` in `[metadata]` suppresses missing-opcode warnings. This signals intentional incomplete coverage.
- **D-10:** Overlapping rules are an error. Two rules that match the same note/input are rejected. No ambiguity in a single tuning file. Different mapping strategies = separate tuning files, not overlapping rules.

### Default Piano Tuning
- **D-11:** Root note = middle C (MIDI 60). Semitone offsets calculated from this.
- **D-12:** Value zone: MIDI 36-59 (C2 through B3) — 2 octaves below middle C. These notes push their semitone offset from root (-24 to -1).
- **D-13:** Operation zone: white keys in the operation range. C major scale notes carry the 19 opcodes. The programmer plays "in key" when computing.
- **D-14:** Musical from day one — the agent will design the specific white-key assignments so that common instruction sequences (push-push-op patterns from Hello World, FizzBuzz) produce pleasant melodic fragments in C major. The layout will be reviewed during planning.
- **D-15:** Black keys in the operation zone: agent's discretion (common constants, aliases, or unmapped).

### Agent's Discretion
- Crate structure decision: tuning engine within `haydn-vm` vs. separate `haydn-tuning` crate
- Specific white-key → opcode assignments for the default piano tuning (optimized for musicality, reviewed during planning)
- Black key assignments in the operation zone
- Internal data structures for the rule engine (rule representation, matching algorithm)
- TOML parsing library choice (e.g., `toml` crate)
- Test organization for the tuning system

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### VM Event Interface (the contract tuning must produce)
- `docs/SPECIFICATION.md` §4 — Event types: `Push(i64)` and `Op(Opcode)`, metadata (ignored in v1)
- `docs/SPECIFICATION.md` §3.4 — Canonical opcode enumeration (19 opcodes that tuning files map to)
- `crates/haydn-vm/src/types.rs` — `Event`, `Opcode` type definitions (the Rust types tuning must produce)

### Prior Phase Decisions
- `.planning/phases/01-language-specification/01-CONTEXT.md` — D-01 through D-06: note-to-value as semitone offsets, tuning owns all mapping logic, event interface contract
- `.planning/phases/02-vm-core/02-CONTEXT.md` — D-05: crate workspace structure, D-10/D-11: event-driven streaming architecture

### Project Context
- `.planning/PROJECT.md` — Core vision, constraints, key decisions
- `.planning/REQUIREMENTS.md` — TUNE-01, TUNE-02, TUNE-03, TUNE-06, TUNE-07, DOC-07 requirements for this phase

### Tuning Schema Examples (temporary reference)
- `.planning/scratch/tuning-schema-examples.md` — Detailed comparison of schema options A/B/C with full examples (created during discussion)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/haydn-vm/src/types.rs`: `Event` enum (`Push(i64)` / `Op(Opcode)`) and `Opcode` enum — these are the output types the tuning engine must produce
- `crates/haydn-vm/src/lib.rs`: `HaydnVm` with `feed(Event)` and `step()` — the consumer interface for tuning output

### Established Patterns
- Cargo workspace with `haydn-vm` as library crate and `haydn` as binary crate
- Zero external dependencies in `haydn-vm` — tuning system may introduce first external dep (`toml` crate for parsing)
- Event-driven architecture: events flow in one-at-a-time via `feed()`, results flow out via `step()`

### Integration Points
- Tuning engine sits between input layer (Phase 4 MIDI / Phase 6 audio) and VM. It receives raw note events and produces `Event` values
- The `Event` type is already defined in `haydn-vm` — tuning engine must produce exactly these types
- Default `piano.toml` will be embedded or bundled with the binary (Phase 9 distribution)

</code_context>

<specifics>
## Specific Ideas

- The performer envisions programming in Haydn as sounding *musical* — common instruction sequences should produce pleasant melodic patterns, not atonal noise. This is core to the project's identity ("the performance IS the program").
- White-key-only opcodes in C major means the default piano tuning sounds diatonic during normal programming. This was a deliberate artistic choice by the user.
- The rule engine's flexibility means a tuning author can create interval-based mappings where the *distance between notes* determines the operation — e.g., perfect fifth = add, perfect fourth = sub. This enables deeply musical programming styles.
- The user wants to hear and evaluate tuning musicality as early as possible. A simulated performer phase (3.1) was discussed to enable audio preview before real MIDI hardware is needed.

</specifics>

<deferred>
## Deferred Ideas

- **Simulated performer (Phase 3.1):** A separate tool/crate that takes a program sequence + tuning file, generates the corresponding note sequence, plays it through computer audio (synth), and feeds it into the VM simultaneously. Enables musicality judgment without MIDI hardware. **Placement decided: Phase 3.1** — use `/gsd-insert-phase 3.1` to add to roadmap.
- **Hot-swap tuning files at runtime** — switch active tuning mid-performance via MIDI CC or CLI command. Phase 4/9 feature.
- **Reserve `haydn-tuning` crate name on crates.io** — consider if tuning becomes a separate crate.
- **Additional trigger types** — velocity-based, sequence-based, zone-with-named-regions. Expandable from the rule engine; add when needed (Phase 7 voice/guitar tunings may drive this).
- **MIDI file export for preview** — generate a MIDI file from a program sequence to hear tuning musicality without real-time audio. Alternative to the sim phase.

</deferred>

---

*Phase: 03-tuning-system*
*Context gathered: 2026-03-29*
