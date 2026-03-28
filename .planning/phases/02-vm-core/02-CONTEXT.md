# Phase 2: VM Core - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the stack-based virtual machine with all 19 opcodes and graceful edge case handling. The VM is a pure computation engine — event-driven, deterministic, no music awareness. It receives `Event` values (`Push(i64)` or `Op(Opcode)`) and executes them per the Haydn Language Specification. This phase delivers `haydn-vm` as a standalone library crate with zero external dependencies.

</domain>

<decisions>
## Implementation Decisions

### Observability Hooks
- **D-01:** Step-by-step core architecture — VM exposes `step()` that processes exactly one event (live or replayed), returning a `StepResult` struct with: operation executed, stack snapshot/delta, output bytes, edge case flags, and replay state.
- **D-02:** `process_event()` convenience wrapper for callers who don't need per-step granularity — internally calls `step()` in a loop until the VM is waiting for live input, returns `Vec<StepResult>`.
- **D-03:** Loop replay pacing is a caller/TUI concern, not a VM concern. The VM always steps one event at a time. Visible vs instant replay is controlled by how fast the caller calls `step()`.

### Edge Case Feedback
- **D-04:** `StepResult` includes an optional `edge_case: Option<EdgeCase>` field (e.g., `StackUnderflow`, `DivisionByZero`, `NegativeAddress`). VM always handles gracefully per spec, but caller knows it happened. TUI decides whether/how to display.

### Crate Structure
- **D-05:** Cargo workspace with `haydn-vm` as standalone library crate (zero audio/MIDI deps) and `haydn` as the CLI binary crate. Clean separation enables future WASM target, embeddable VM, and isolated testing.
- **D-06:** Reserve `haydn` and `haydn-vm` on crates.io early. (In progress — requires email verification on crates.io.)

### I/O Integration
- **D-07:** Output bytes included in `StepResult` (`Option` field, `None` for non-output steps). No shared buffers or callbacks.
- **D-08:** Input via `vm.provide_input(bytes)` method feeding an internal `VecDeque<u8>`. The `read` opcode pops from buffer or returns `0` if empty. Non-blocking per spec.
- **D-09:** Agent's discretion on exact `StepResult` field types (e.g., `SmallVec` optimization for output bytes, enum variants for edge cases).

### Future-Proofing (confirmed during discussion)
- **D-10:** The event-driven `step()` architecture IS a streaming architecture — no "load entire program" step exists. Events flow in one-at-a-time, results flow out one-at-a-time.
- **D-11:** Phrase recognition (multiple notes with compound meaning) and chord detection (simultaneous notes) are tuning layer concerns (Phase 3). The VM sees only the resulting `Event` stream after the tuning layer resolves musical gestures. VM design places zero constraints on tuning complexity.
- **D-12:** Multi-instrument support is a runtime/caller concern. Since `haydn-vm` is a stateless library crate with no global state, callers can instantiate multiple independent VMs trivially (`let mut vm1 = HaydnVm::new(); let mut vm2 = HaydnVm::new();`). Ensemble programming, instrument-specific tunings, and merged streams are all caller decisions.
- **D-13:** Non-blocking I/O (`read` returns 0 when empty) ensures the VM never stalls — critical for live performance and for inter-VM communication patterns.

### Agent's Discretion
- Exact `StepResult` struct field types and naming
- Internal data structure choices (e.g., `Vec` vs `SmallVec` for loop buffers)
- Whether `step()` returns owned or borrowed data
- Test organization within the `haydn-vm` crate

</decisions>

<specifics>
## Specific Ideas

- **Chord-triggered instant replay mode:** Performer plays a specific chord or reserved note to trigger instant replay of the current loop iteration (skipping the step-by-step pacing). Deferred to Phase 3 (tuning) / Phase 5 (TUI) — the VM architecture supports this by design since the caller controls replay pacing.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### VM Specification
- `docs/SPECIFICATION.md` — Complete language spec: 19 opcodes, event interface (`Push`/`Op`), loop pedal state machine, edge case behavior, I/O model, Turing completeness proof
- `docs/SPECIFICATION.md` §2 — Computational model: i64 values, unbounded stack, addressable memory, I/O streams
- `docs/SPECIFICATION.md` §3 — Full opcode table with stack effects and edge case behavior
- `docs/SPECIFICATION.md` §4 — Event interface: `Push(i64)` and `Op(Opcode)`, metadata (ignored in v1)
- `docs/SPECIFICATION.md` §5 — Loop pedal model: Recording/Replaying/Normal states, full state transition table

### Project Context
- `.planning/PROJECT.md` — Core vision, constraints, key decisions
- `.planning/REQUIREMENTS.md` — VM-01 through VM-08 requirements for this phase
- `.planning/phases/01-language-specification/01-CONTEXT.md` — Phase 1 decisions (D-01 through D-14) — contains foundational choices about note-to-value mapping, the tuning layer separation, and the loop pedal model

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — this is the first code phase. The repo contained only documentation prior to this phase.

### Established Patterns
- Workspace structure created during crate reservation: root `Cargo.toml` (workspace), `crates/haydn-vm/` (library), `crates/haydn/` (binary). All Phase 2 implementation goes in `crates/haydn-vm/`.

### Integration Points
- `haydn-vm` is consumed by `haydn` (CLI binary, Phase 5+) and by test harnesses (this phase)
- The `Event` type defined here becomes the contract that the tuning layer (Phase 3) must produce
- `StepResult` becomes the contract that the TUI (Phase 5) consumes for visualization

</code_context>

<deferred>
## Deferred Ideas

- **Chord-triggered instant replay** — Performer gesture to skip visible replay pacing → Phase 3 (tuning) / Phase 5 (TUI)
- **Reserve `haydn-tuning` crate name** — Consider during Phase 3
- **Full crate ecosystem naming** (`haydn-audio`, `haydn-midi`, etc.) — Revisit during Phase 9 (Distribution)
- **EventMetadata (velocity, duration_ms)** — Spec defines it for forward compatibility; v1 VM ignores it. Revisit in v2 milestone.

</deferred>

---

*Phase: 02-vm-core*
