# Phase 1: Language Specification — Research

**Researched:** 2026-03-28
**Discovery Level:** 2 (Standard — novel live-input loop semantics require careful analysis)
**Phase Requirements:** SPEC-01 through SPEC-08, DOC-06

## Executive Summary

Phase 1 produces a formal specification document — no code. The key research areas are: (1) Brainfuck instruction mapping for the Turing completeness proof, (2) stack-based VM edge case conventions, (3) the novel "loop pedal" control flow model that adapts Brainfuck's `[ ]` brackets to a live-input stream, and (4) spec document structure conventions from the esolang tradition.

## 1. Brainfuck Reference — Reduction Target

Haydn's TC proof maps Brainfuck instructions to Haydn VM opcodes. The mapping must be complete and unambiguous.

### Brainfuck Instruction Set (8 instructions)

| BF | Semantics | Haydn Equivalent |
|----|-----------|------------------|
| `>` | Move data pointer right | `store` / `load` (memory addressing) |
| `<` | Move data pointer left | `store` / `load` (memory addressing) |
| `+` | Increment cell at pointer | `push(1)` then `add` |
| `-` | Decrement cell at pointer | `push(1)` then `sub` |
| `.` | Output byte at pointer | `print_char` |
| `,` | Input byte to pointer | `read` |
| `[` | If cell at pointer is 0, jump past matching `]` | `loop_start` (per D-09: check top of stack) |
| `]` | If cell at pointer is nonzero, jump back to matching `[` | `loop_end` (per D-09: conditional replay) |

### Key Reduction Requirements

1. **Memory simulation:** Brainfuck has a flat tape; Haydn has a stack + addressable memory (`store`/`load`). Use a convention: one memory cell per BF tape cell, with a "pointer" value on the stack tracking current position.
2. **Conditional branching:** BF's `[ ]` requires checking the current cell. In Haydn: `load` current cell to stack top, `loop_start`/`loop_end` checks stack top per D-09.
3. **Nested loops:** BF allows arbitrary `[ ]` nesting. Haydn's loop stack (D-08) maps directly.
4. **The proof must show:** Every BF program can be mechanically translated to a sequence of Haydn `Push(value)` and `Op(opcode)` events that produces identical I/O behavior.

### Proof Structure Recommendation

Use **compilation-style reduction**: define a function `compile(bf_program) → haydn_event_sequence` that maps each BF instruction to a fixed Haydn event subsequence. Show: (a) the mapping is total, (b) I/O behavior is preserved, (c) halting behavior is preserved. This is standard for esolang TC proofs.

## 2. Stack-Based VM Specification Conventions

### Standard Opcode Categories

Based on established stack-machine traditions (Forth, PostScript, JVM bytecode, WebAssembly):

**Stack manipulation:** push, dup, swap, drop, rotate (over, nip, tuck are optional enrichments)
**Arithmetic:** add, sub, mul, div, mod (all binary — pop two, push result)
**Comparison:** eq, gt, lt (pop two, push boolean as 0/1)
**Control flow:** loop_start, loop_end (per decisions D-07–D-10)
**I/O:** print_num, print_char, read
**Memory:** store, load (address-based, separate from stack)

### Minimum Opcode Set for TC

The absolute minimum for Turing completeness (via BF reduction):
- `push`, `add`, `sub` (value manipulation)
- `store`, `load` (tape simulation)
- `loop_start`, `loop_end` (conditional branching)
- `print_char`, `read` (I/O to match BF)

The full set (with `dup`, `swap`, `drop`, `rotate`, `mul`, `div`, `mod`, `eq`, `gt`, `lt`, `print_num`) is richer than needed for TC but makes the language practically usable for interesting programs.

### Edge Case Conventions (SPEC-02)

Standard approaches from existing stack languages:

| Edge Case | Common Approaches | Recommendation |
|-----------|-------------------|----------------|
| Stack underflow (pop from empty) | (a) Push 0, (b) Error/halt, (c) No-op | **Push 0** — graceful, keeps performance going. Musicians shouldn't crash mid-performance. |
| Division by zero | (a) Push 0, (b) Push MAX_INT, (c) Error | **Push 0** — simple, predictable, matches "graceful" goal |
| Invalid memory address | (a) Return 0, (b) Auto-extend, (c) Modular wrap | **Return 0 for reads, no-op for writes to negative** — safe default |
| Integer overflow | (a) Wrap (modular), (b) Saturate, (c) BigInt | **Wrapping arithmetic (i64)** — standard, predictable |
| Empty stack for loop_end check | (a) Treat as 0 (exit loop), (b) Error | **Treat as 0** — exits loop, consistent with "empty = zero" |
| print_char with value > 127 | (a) Modular (val % 128), (b) UTF-8, (c) Ignore | **Modular (val % 256 as byte)** — simple, matches BF tradition |
| read with no input available | (a) Push 0, (b) Push -1, (c) Block | **Push 0** — non-blocking, live performance can't wait |

### Value Representation

**i64 (signed 64-bit integer)** is the standard recommendation:
- Signed: needed for negative semitone offsets (D-02)
- 64-bit: large enough for any practical musical program
- Wrapping: standard Rust wrapping_add/sub/mul semantics

## 3. Live-Input Loop Semantics — Novel Design Analysis

This is the most novel aspect of Haydn's spec. Traditional languages have a fixed program; Haydn's "program" arrives as a real-time event stream. The loop mechanism must work in this context.

### The Loop Pedal Model (D-07 through D-10)

**State machine for loops:**

```
NORMAL → (loop_start event) → RECORDING
RECORDING → (events accumulate in loop buffer) → RECORDING
RECORDING → (loop_end event) → CHECK
CHECK → (stack top ≠ 0) → REPLAYING (replay buffer from start)
CHECK → (stack top = 0) → NORMAL (discard buffer, continue)
REPLAYING → (buffer exhausted) → CHECK (re-check stack top)
```

**Nesting (D-08):**
- Each `loop_start` pushes a new buffer onto the loop stack
- Each `loop_end` pops the current buffer and replays/discards
- During replay, if a `loop_start` is encountered, a NEW inner loop begins recording
- This maps directly to BF's nested `[ ]` semantics

### Critical Spec Questions to Resolve

1. **What happens if `loop_end` is received with no active loop?** Recommendation: **No-op** (ignore it). The performer made a mistake; don't crash.

2. **What happens if the program ends (input stops) inside a loop?** Recommendation: **Discard all open loop buffers.** The "program" is just the events received. If you stop playing inside a `[`, it acts as if you played enough `]` to close all open loops with stack-top = 0.

3. **During replay, does the VM accept new live input?** Recommendation: **No.** During replay of a loop body, the VM processes only buffered events. Live input is queued and processed after replay completes (or loop exits). This avoids interleaving live events with replayed events, which would be chaotic and unmusical.

4. **Can nested loops have independent replay counts?** Yes — each loop buffer is independent. Inner loop replays until its own stack-top check fails, then outer loop body continues.

5. **Maximum nesting depth?** Spec should define a reasonable limit (e.g., 256) to prevent unbounded memory use. Stack overflow on loop nesting → exit all loops, return to NORMAL.

### Formal Loop Semantics

The spec should define loops in terms of:
- **Loop stack**: `Vec<Vec<Event>>` — stack of event buffers
- **Loop state enum**: `Normal | Recording | Replaying(position)`
- **Transition rules**: One rule per (state, event) pair
- **Invariants**: "Loop nesting depth ≤ MAX_LOOP_DEPTH", "Replay does not accept live input"

## 4. Event Interface Contract (D-04, D-05, D-06)

The spec defines the boundary between the tuning system and the VM.

### Event Types

```
Event = Push(value: i64) | Op(opcode: Opcode)
```

Where `Opcode` is one of the VM operations (dup, swap, drop, rotate, add, sub, mul, div, mod, eq, gt, lt, loop_start, loop_end, print_num, print_char, read, store, load).

Note: `Push` is NOT an opcode — it's a separate event type. This is a key distinction: tuning files produce `Push(value)` for note-as-value and `Op(opcode)` for operation triggers.

### Optional Metadata (D-12)

Events MAY carry metadata:
```
EventMetadata = { velocity: Option<u8>, duration_ms: Option<u32>, ... }
```

v1 VM ignores metadata. Metadata exists for forward compatibility (v2 velocity-based operations) and for tuning files that want to use metadata in their own mapping logic.

### Event Interface Summary for Spec

The spec MUST define:
1. The `Event` enum (Push + Op) — exhaustive
2. The `Opcode` enum — exhaustive (all VM operations)
3. The `EventMetadata` struct — extensible
4. A semantic contract: "The VM processes events sequentially. Each event either pushes a value or executes an operation. The VM does not interpret the source of events."

## 5. Silence and Timing (D-11, D-13, D-14)

### Spec-Level Semantics

- **Silence = absence of events.** The VM is event-driven; no events means no computation. The VM simply waits.
- **The spec does NOT define timing.** There is no "clock." Events are processed in order of receipt, with no timing semantics.
- **Sustained notes:** The spec defines no special behavior. If the input layer sends events for a sustained note, the VM processes them like any other events. The tuning file decides what (if anything) to emit for sustained notes (D-13).

### What the Spec Explicitly Defers

- Tempo, rhythm, dynamics (D-14) — v2
- Time-based operations — v2
- Multi-channel/multi-performer — v2

## 6. Spec Document Structure — Esolang Convention

Esolang specifications typically follow this structure:

1. **Introduction / Overview** — What the language is, key concept
2. **Computational Model** — Stack, memory, I/O model
3. **Instruction Set** — Each opcode with precise semantics, before/after stack state
4. **Control Flow** — How loops/jumps work
5. **Event Interface** — How instructions enter the VM (unique to Haydn)
6. **Edge Cases** — Exhaustive table of error conditions and handling
7. **Turing Completeness** — Proof via reduction
8. **Examples** — Simple programs showing instruction sequences

### Format Recommendation

Markdown document with:
- Tables for instruction reference (opcode, stack effect, description)
- Pseudocode for complex semantics (loop state machine)
- Formal notation for the TC proof (but not overly academic — esolang tradition is accessible)

## 7. Opcode Catalog — Full Reference for Spec

| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `dup` | `a -- a a` | Duplicate top of stack |
| `swap` | `a b -- b a` | Swap top two elements |
| `drop` | `a --` | Remove top of stack |
| `rotate` | `a b c -- b c a` | Rotate top three elements |
| `add` | `a b -- (a+b)` | Addition |
| `sub` | `a b -- (a-b)` | Subtraction (second minus top) |
| `mul` | `a b -- (a*b)` | Multiplication |
| `div` | `a b -- (a/b)` | Integer division (truncated toward zero) |
| `mod` | `a b -- (a%b)` | Modulo (sign follows dividend) |
| `eq` | `a b -- (a==b)` | Equality: push 1 if equal, else 0 |
| `gt` | `a b -- (a>b)` | Greater than: push 1 if a>b, else 0 |
| `lt` | `a b -- (a<b)` | Less than: push 1 if a<b, else 0 |
| `loop_start` | `--` | Begin loop recording |
| `loop_end` | `a -- (conditionally)` | Check top: nonzero → replay, zero → exit loop |
| `print_num` | `a --` | Print top as decimal integer |
| `print_char` | `a --` | Print top as ASCII/byte character |
| `read` | `-- a` | Read one byte of input (0 if unavailable) |
| `store` | `val addr --` | Store val at memory address addr |
| `load` | `addr -- val` | Load value from memory address addr |

**Note on `sub`/`div`/`mod` operand order:** The spec MUST be explicit. Convention: `a b sub` means `a - b` where `b` is top of stack and `a` is second. This matches Forth convention. The spec must state: "Binary operations pop the top of stack as the RIGHT operand and the second element as the LEFT operand."

## 8. Planning Implications

### Document Scope
This phase produces ONE artifact: the formal specification document (`docs/SPECIFICATION.md` or similar). It's a pure documentation phase — no Rust code, no Cargo.toml, no tests.

### Natural Task Split for Planning
1. **VM core spec** — Computational model, value types, stack/memory, opcode table with all edge cases (SPEC-01, SPEC-02)
2. **Event interface + control flow** — Event types, loop state machine, silence semantics (SPEC-04, SPEC-05, SPEC-06, SPEC-07, SPEC-08)
3. **TC proof + publication** — Brainfuck reduction proof, examples, final document assembly (SPEC-03, DOC-06)

### Risk: Live-Input Loop Semantics
The loop pedal model (D-07–D-10) is the most novel and complex part. The spec must handle:
- Unmatched loop boundaries
- Input during replay
- Nesting limits
- Interaction with all other opcodes during replay (do store/load work? yes. does read work? spec must say)

Recommendation: The loop state machine should be specified with a complete transition table, not prose.

## Sources

- Brainfuck language specification (Daniel B. Cristofani): 8-instruction tape-based TC language, canonical esolang
- Forth-83 Standard and ANS Forth: stack effect notation conventions, operand ordering
- Esolang wiki conventions: spec document structure, TC proof expectations
- WebAssembly specification: modern stack machine formal specification patterns
- JVM Specification (Chapter 6): instruction set documentation conventions

---

*Phase: 01-language-specification*
*Research completed: 2026-03-28*
