# Phase 2: VM Core - Discussion Log

**Date:** 2026-03-28
**Mode:** Interactive (discuss)
**Areas discussed:** 4 of 4 identified

---

## Gray Areas Identified

1. **Observability hooks** — How does the VM expose internal state changes for visualization?
2. **Edge case feedback** — How does the caller learn when edge cases (stack underflow, div-by-zero) occur?
3. **Crate structure** — How to organize the Rust workspace?
4. **I/O integration style** — How do output and input flow between VM and caller?

User selected: **all**

---

## Area 1: Observability Hooks

### Options Presented
| Option | Description | Tradeoff |
|--------|-------------|----------|
| A | `process_event()` returns `StepResult` with summary | Simple API, but loop replay is invisible — replayed events appear as one atomic step |
| B | `step()` processes one event at a time, returns `StepResult` per event | Caller sees every replayed event individually; controls pacing; internally simpler (one code path) |
| C | Callback/observer pattern | Flexible but complex; Rust lifetime issues |

### Discussion
- User asked for layman explanation of loop replay visibility
- Agent explained: Option A = "instant replay" (like a musician loop pedal in fast-forward), Option B = "visible replay" (loop pedal plays back at normal speed where you can watch each note)
- User asked engineering tradeoff question: "what is the advantage?"
- Agent explained: Step-by-step (B) is actually *simpler* internally — one code path for all events regardless of live vs replayed. Option A requires "am I replaying?" conditional logic.
- User asked about caller-controlled pacing: "so the caller can choose instant or visible?"
- Agent confirmed: Yes — caller calls `step()` fast (tight loop) for instant, adds delays for visible. VM doesn't know or care.
- User suggested "chord-triggered instant replay" — a performer gesture that switches from visible to instant replay mid-loop. Agent confirmed architecture supports this, deferred the mapping to Phase 3/5.

### Decision
**Option B (step-by-step)** with a `process_event()` convenience wrapper.
- **D-01:** `step()` = one event, one `StepResult`
- **D-02:** `process_event()` = calls `step()` until waiting for live input, returns `Vec<StepResult>`
- **D-03:** Replay pacing = caller concern

---

## Area 2: Edge Case Feedback

### Options Presented
| Option | Description |
|--------|-------------|
| 1 | Separate `vm.last_edge_case()` query method |
| 2 | `StepResult` includes `edge_case: Option<EdgeCase>` field |
| 3 | Callback on edge case |

### Discussion
Minimal — user immediately selected Option 2 as the cleanest approach.

### Decision
**Option 2** — `StepResult` includes optional edge case flag.
- **D-04:** `Option<EdgeCase>` enum with variants like `StackUnderflow`, `DivisionByZero`, `NegativeAddress`

---

## Area 3: Crate Structure

### Options Presented
| Option | Description |
|--------|-------------|
| A | Single crate, modules only | Simplest start, harder to split later |
| B | Workspace: `haydn-vm` library + `haydn` binary | Clean separation, embeddable VM, independent testing |

### Discussion
- User chose workspace immediately
- User asked about reserving crate names on crates.io early
- Agent recommended reserving `haydn` and `haydn-vm` now, `haydn-tuning` during Phase 3, others during Phase 9

### Decision
**Option B** — Cargo workspace.
- **D-05:** `haydn-vm` = standalone library (zero deps), `haydn` = CLI binary
- **D-06:** Reserve both names on crates.io

---

## Area 4: I/O Integration

### Options Presented
| Aspect | Approach |
|--------|----------|
| Output | Bytes included in `StepResult` as `Option` field |
| Input | `vm.provide_input(bytes)` feeding internal `VecDeque<u8>`; `read` pops or returns 0 |

### Discussion
- User asked for comprehensive evaluation of idiomatic-ness, naming, and long-term viability
- Agent confirmed the approach is standard Rust (no callbacks, no shared state, no channels — just values in, values out)
- User asked whether "esoteric language" framing is correct — agent confirmed yes, Haydn sits in the tradition of Brainfuck/Piet/Befunge
- Agent provided full analysis of crate naming strategy (`haydn` + `haydn-vm` now, others later)
- User asked whether I/O design restricts future capabilities (streaming, phrases, chords, multi-instrument, parallelism)
- Agent confirmed: event-driven step architecture IS streaming; phrase/chord recognition is tuning layer; multi-instrument = multiple VM instances (trivial with library crate); non-blocking I/O ensures no stalls

### Decision
- **D-07:** Output in `StepResult` (Option field)
- **D-08:** Input via `provide_input()` → `VecDeque<u8>` → `read` pops or returns 0
- **D-09:** Agent discretion on exact types
- **D-10 through D-13:** Future-proofing confirmed (see CONTEXT.md)

---

*Phase: 02-vm-core*
