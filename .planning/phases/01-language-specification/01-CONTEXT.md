# Phase 1: Language Specification - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Define the complete, unambiguous formal specification of the Haydn language. This covers the VM instruction set, execution semantics, event interface contract, edge cases, and Turing completeness proof. The spec defines the computational engine — not the musical mappings (those are Phase 3: Tuning System).

</domain>

<decisions>
## Implementation Decisions

### Note-to-Value Mapping
- **D-01:** Notes produce values as **semitone offsets from a configurable root note** (set in the tuning file). Root = 0, one semitone up = 1, one semitone down = -1.
- **D-02:** **Negative offsets are allowed** — notes below the root produce negative stack values. Bidirectional value range.
- **D-03:** The spec defines the abstract semantics ("a note produces a value equal to its semitone distance from the configured root"). The tuning file sets the root. No fixed absolute MIDI number mapping at the spec level.

### Interval-to-Operation Design (Spec vs Tuning Boundary)
- **D-04:** The spec defines **only the VM and the event interface**. The spec does NOT define how musical input maps to operations — that's entirely the tuning file's domain.
- **D-05:** The event interface consists of two event types: `Push(value)` and `Op(opcode)`. Tuning files produce these events from musical input however they see fit (interval-based, alternating, modal, velocity-based, etc.).
- **D-06:** This means SPEC-05 (note-as-value) and SPEC-06 (interval-as-operation) are about specifying the **interface contract** between tuning files and the VM, not fixed musical grammar rules.

### Live-Input Control Flow
- **D-07:** **Loop pedal model** — the VM has loop-start and loop-end operations. Loop-start begins recording events; loop-end conditionally replays the recorded segment.
- **D-08:** **Nested loops** supported via a loop stack. Starting a new loop inside an existing one nests automatically. Required for Turing completeness.
- **D-09:** **Conditional replay (Brainfuck-style)** — at loop-end, check top of stack: nonzero replays the segment, zero exits the loop and continues. This enables the TC proof via direct Brainfuck reduction.
- **D-10:** No forward jumps or goto in v1. All control flow is loop-based. The performer triggers loop boundaries; the VM handles replay internally.

### Silence and Timing Semantics
- **D-11:** **Silence is a no-op at the spec/VM level.** The VM simply receives no events and waits. Tuning files may optionally emit events on silence timeout if they want silence to carry meaning.
- **D-12:** **Events carry optional metadata** (velocity, duration, etc.). The spec defines that events CAN have metadata, but standard VM opcodes do not require it. Tuning files decide whether and how to use metadata (e.g., velocity-based operation selection).
- **D-13:** **Sustained/held notes can trigger additional events** — this is tuning-file-driven. The input layer detects note sustain and the tuning file decides whether to emit additional events (e.g., dup, modifier, no-op).
- **D-14:** **Tempo, dynamics (crescendo/decrescendo), rhythm, ritardando are deferred to v2.** v1 responds to WHAT you play and optional velocity, not HOW FAST or with what dynamic arc.

### Agent's Discretion
- Spec document format and structure (markdown, sections, formal notation style)
- Exact Turing completeness proof approach (Brainfuck reduction is decided, but proof presentation style is flexible)
- Edge case handling specifics (stack underflow, division by zero, invalid memory) — general approach is "graceful handling per spec," agent determines exact behavior for each case
- How the event interface contract is formally expressed (type signatures, pseudocode, prose)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above.

### Project Context
- `.planning/PROJECT.md` — Core vision, constraints, key decisions
- `.planning/REQUIREMENTS.md` — SPEC-01 through SPEC-08, DOC-06 requirements for this phase

### Prior Art (for reference during spec writing)
- Brainfuck language specification — direct reduction target for Turing completeness proof
- Esolang wiki conventions — for spec document style and completeness expectations

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — this is Phase 1, no code exists yet. Project is a blank Rust repository.

### Established Patterns
- None — patterns will be established starting from Phase 2 (VM Core).

### Integration Points
- The language spec produced here is the foundational document for all subsequent phases. Phase 2 (VM Core) implements the VM directly from this spec. Phase 3 (Tuning System) implements the event interface contract defined here.

</code_context>

<specifics>
## Specific Ideas

- The performer envisions an **exploratory discovery** experience — "play around and discover what happens" rather than deterministic demos. Tuning files + performance descriptions are shareable like "level seeds."
- The loop pedal model was chosen because it mirrors how musicians already think about repetition — "record a phrase, replay it." The VM's loop-start/loop-end maps directly to this mental model.
- The user expressed strong excitement about velocity, dynamics, and expressive musical features (crescendo, ritardando, etc.). These are deferred to v2 but should be designed with forward compatibility — the optional metadata on events is the v1 hook for this.
- The separation of spec (VM) from musical grammar (tuning files) was a deliberate artistic choice: "music is art" — the tuning file should fit the musician's style rather than the spec imposing a single grammar on all performers.
- Community sharing angle: performers share tuning file + performance description combos. Like sharing level seeds in a game. Reproducibility comes from shared tuning files, not from the spec enforcing a fixed mapping.

</specifics>

<deferred>
## Deferred Ideas

- **Multi-performer / ensemble mode** — performer expressed interest in loopers creating overlapping layers that Haydn could interpret as multiple streams. Explicitly v2+ (multi-performer is out of scope for v1).
- **Real-time loop overlay interpretation** — recognizing previously-seen patterns from looped audio. v2+ feature.
- **Tempo/rhythm/dynamics awareness** — crescendo, decrescendo, ritardando as semantic elements. Deferred to v2; v1 carries velocity as optional metadata as a bridge.
- **Velocity as a rich expressive dimension** — full velocity-based operation modes. v1 supports velocity as optional event metadata; richer velocity semantics are v2.

</deferred>

---

*Phase: 01-language-specification*
*Context gathered: 2026-03-28*
