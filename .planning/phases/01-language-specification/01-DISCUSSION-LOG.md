# Phase 1: Language Specification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 1-Language Specification
**Areas discussed:** Note-to-value mapping, Interval-to-operation design, Live-input control flow, Silence and timing semantics

---

## Note-to-Value Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Absolute MIDI numbers | MIDI note number directly — C4=60, C#4=61, etc. Simple, absolute, 0-127 range | |
| Relative to root note | Semitone offset from a configurable root note — root=0, one semitone up=1, etc. | ✓ |
| Pitch class only | Only the pitch class matters (0-11), octave is ignored — smaller value range | |
| Tuning file defines values | Notes map to arbitrary values per config, no built-in rule | |

**User's choice:** Relative to root note
**Notes:** User preferred exploratory discovery over deterministic demos. Liked the idea that performers share tuning file + performance description combos (like level seeds). Wanted bidirectional offsets — notes below root produce negative values.

### Follow-up: Negative offsets

| Option | Description | Selected |
|--------|-------------|----------|
| Negative offsets (bidirectional) | Notes below root produce negative stack values — richer value range | ✓ |
| Non-negative only (upward from root) | Below-root notes clamp to 0 or are ignored — only upward offsets | |

**User's choice:** Negative offsets (bidirectional)

---

## Interval-to-Operation Design

| Option | Description | Selected |
|--------|-------------|----------|
| Signed interval distance | Semitone distance between consecutive notes selects opcode, direction matters | |
| Absolute interval distance | Only absolute semitone distance matters, direction ignored | |
| Alternating note roles | First note pushes, second note operates, alternating rhythm | |
| Modal (value mode vs operation mode) | Special trigger switches between value mode and operation mode | |

**User's choice:** None of the above — user wanted maximum artistic flexibility

### Follow-up: Where should musical grammar live?

| Option | Description | Selected |
|--------|-------------|----------|
| Machine + canonical grammar (overridable) | Spec defines one canonical musical grammar, tuning files can override | |
| Machine only, grammar in tuning files | Spec defines only the VM. ALL musical grammar lives in tuning files. | ✓ |
| Machine + fixed grammar, tuning tweaks params | Spec defines both strictly, tuning files only tweak parameters | |

**User's choice:** Machine only, grammar in tuning files
**Notes:** User said "music is art" — the tuning file should fit the musician's style. The spec shouldn't impose a single grammar on all performers. This means SPEC-05 and SPEC-06 become about the event interface contract, not fixed musical rules.

---

## Live-Input Control Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Label-based (drop markers, jump to them) | Label event marks a point in event stream, jump goes back to named label | |
| Repeat-last-N (backward replay only) | Loop event means "repeat the last N events," no forward jumps | |
| Loop pedal model (record/replay segments) | Loop-start begins recording, loop-end replays recorded segment | ✓ |
| No VM-level loops (performer repeats physically) | No jump/loop in VM, performer physically repeats phrases | |

**User's choice:** Loop pedal in the VM
**Notes:** User initially gravitated toward "performer repeats physically" and mentioned using actual looper pedals with multiple layers. Those ideas (loop overlay, ensemble, multi-track) were noted as v2+ and deferred. The loop pedal VM model was chosen as the v1 approach because it's musical, intuitive, and TC-capable.

### Follow-up: Loop nesting

| Option | Description | Selected |
|--------|-------------|----------|
| Single loop (most recent only) | Only most recent loop segment can be active | |
| Nested loops (stack of loop segments) | Loops can nest — loop-end replays innermost active segment | ✓ |

**User's choice:** Nested loops (agent recommendation accepted)
**Notes:** Nested loops required for Turing completeness. Also mirrors musical structure (verse within song within set). User asked agent to recommend; agent chose nested loops for TC requirement and musical naturalness.

### Follow-up: Loop condition

| Option | Description | Selected |
|--------|-------------|----------|
| Conditional replay (check top of stack) | Pop top of stack: nonzero replays, zero exits. Brainfuck-style. | ✓ |
| Replay-then-check (do-while) | Always replay once, then check condition | |
| Fixed count replay | Replay N times based on top-of-stack value | |

**User's choice:** Conditional replay (agent recommendation accepted)
**Notes:** User asked which is most musical and flexible. Agent recommended conditional replay for maximum flexibility, musical naturalness (repeat "until it feels right"), and straightforward Brainfuck reduction for TC proof.

---

## Silence and Timing Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Silence = wait (no-op) | VM just waits, nothing happens until next note | ✓ |
| Silence = event after timeout | Silence triggers a special event after configurable timeout | |
| Silence = structural break | Silence ends current loop segment or resets state | |

**User's choice:** Silence = no-op at spec level
**Notes:** Consistent with the "spec defines the machine, tuning defines the music" architecture. Tuning files can optionally emit events on silence timeout.

### Follow-up: Timing and velocity

| Option | Description | Selected |
|--------|-------------|----------|
| Metadata-aware events, dynamics deferred | Events carry optional metadata (velocity, duration). Tuning files decide usage. Tempo/dynamics v2. | ✓ |
| Pure notes only, all dynamics v2 | Events are pure note identity only — no metadata | |

**User's choice:** Metadata-aware events, dynamics deferred
**Notes:** User expressed strong interest in velocity, tempo, crescendo, decrescendo, ritardando — "I'd really like velocity to mean something." These are deferred to v2 per existing Out of Scope decision, but velocity is available as optional event metadata in v1 as a bridge. User was satisfied with this compromise.

### Follow-up: Sustained notes

| Option | Description | Selected |
|--------|-------------|----------|
| Single event (hold = no effect) | Held note = single event, no additional effect | |
| Sustained event (tuning-file-driven) | Held note can trigger additional events, tuning file decides | ✓ |

**User's choice:** Sustained event (tuning-file-driven)

---

## Agent's Discretion

- Spec document format and structure
- Exact Turing completeness proof presentation
- Edge case handling specifics (underflow, div-by-zero, invalid memory)
- Event interface formal expression style

## Deferred Ideas

- Multi-performer / ensemble mode with looper overlay interpretation (v2+)
- Real-time loop overlay interpretation / pattern recognition (v2+)
- Tempo/rhythm/dynamics awareness — crescendo, decrescendo, ritardando (v2)
- Full velocity-based operation modes (v2)
