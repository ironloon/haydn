# Haydn Language Specification

**Version:** 1.0
**Date:** 2026-03-28
**License:** MIT

> Haydn is an esoteric programming language where live musical performance IS the program.

**Abstract.** Haydn is an esoteric programming language in which the performer's live musical input — via MIDI, microphone, or any event source — constitutes the program. This document specifies the Haydn virtual machine: a stack-based, event-driven computation engine with 19 opcodes, signed 64-bit integer values, addressable memory, and a loop pedal control flow model. Haydn is proven Turing-complete via reduction from Brainfuck. The VM is deterministic, never panics, and defines behavior for every edge case — because the performer's experience must be continuous.

## 1. Introduction

Haydn is an esoteric programming language in the tradition of Brainfuck, Piet, and Befunge — deliberately unusual but computationally serious. What makes Haydn unique is that there is no source code file. The performer's musical input — via MIDI controller, microphone, or any other event source — IS the program, interpreted and executed in real time.

The Haydn system consists of two layers:

1. **The Virtual Machine (this specification):** A stack-based computation engine that receives a stream of events and executes them deterministically. The VM knows nothing about music.
2. **Tuning Files (separate concern):** Configuration files that map musical gestures to VM events. A pianist, vocalist, or guitarist each uses a different tuning file, but they all drive the same VM.

This specification defines layer 1: the VM semantics, instruction set, event interface, control flow, and timing model. Everything a developer needs to implement a conforming Haydn interpreter is here. Musical input mapping is intentionally excluded — that belongs to the tuning file specification.

## 2. Computational Model

### 2.1 Values

All values in Haydn are **signed 64-bit integers** (`i64`).

- Range: −9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
- Arithmetic overflow and underflow use **wrapping** (modular) semantics. There is no overflow trap or saturation.

### 2.2 Stack

The primary data structure is an **unbounded LIFO stack** of `i64` values.

- Initially empty.
- There is no maximum stack depth. (Implementations may impose practical limits, but the specification defines no upper bound.)
- Operations that require values from an empty or insufficient stack have defined edge case behavior (see §3).

### 2.3 Memory

Haydn provides an **addressable array of `i64` values**, indexed by non-negative `i64` addresses.

- Initially, all memory locations contain `0`.
- Addresses are `i64` values taken from the stack. Valid addresses are non-negative integers (0, 1, 2, ...).
- Negative addresses are **invalid**. Reads from negative addresses return `0`. Writes to negative addresses are no-ops.
- There is no fixed upper bound on memory addresses. (Implementations may impose practical limits.)

### 2.4 I/O

- **Output stream:** A byte stream. The VM writes bytes via `print_num` and `print_char` operations.
- **Input stream:** A byte stream. The VM reads bytes via the `read` operation. If no input is available, `read` produces `0`. Input is **non-blocking** — the VM never waits for input.

### 2.5 VM State

The complete VM state is the tuple:

```
(stack, memory, loop_state, output_buffer, input_buffer)
```

All state is initialized to empty/zero at VM creation. The VM is entirely deterministic: given the same initial state and the same event sequence, it produces the same output.

## 3. Instruction Set

### 3.1 Operand Ordering

**Binary operations pop the top of stack as the RIGHT operand and the second element as the LEFT operand.** For `a b sub`, `b` is popped first (right/top), `a` is popped second (left/second). The result is `a − b`.

This matches standard Forth convention: the value most recently pushed is the right-hand side of the operation.

### 3.2 Edge Case Philosophy

Haydn is designed for live performance. The VM **never panics**, **never halts on error**, and **never produces undefined behavior**. Every edge case has a defined, deterministic result. The performer's experience must be continuous — errors are absorbed gracefully.

When an operation lacks sufficient operands, missing values are treated as `0` unless otherwise specified in the opcode table below.

### 3.3 Opcode Table

Stack effects use Forth notation: `before -- after` where the rightmost value is the top of stack.

| Opcode | Stack Effect | Description | Edge Case Behavior |
|--------|-------------|-------------|-------------------|
| `dup` | `a -- a a` | Duplicate top of stack | Empty stack: push `0` |
| `swap` | `a b -- b a` | Swap top two elements | Fewer than 2 elements: no-op |
| `drop` | `a --` | Remove top of stack | Empty stack: no-op |
| `rotate` | `a b c -- b c a` | Rotate top three elements — third element moves to top | Fewer than 3 elements: no-op |
| `add` | `a b -- (a+b)` | Add | Fewer than 2 elements: missing operands treated as `0`. Overflow: wrapping. |
| `sub` | `a b -- (a-b)` | Subtract. `b` is top, `a` is second. Result is `a` minus `b`. | Fewer than 2 elements: missing operands treated as `0`. Overflow: wrapping. |
| `mul` | `a b -- (a*b)` | Multiply | Fewer than 2 elements: missing operands treated as `0`. Overflow: wrapping. |
| `div` | `a b -- (a/b)` | Integer division, truncated toward zero | Division by zero: push `0`. Fewer than 2 elements: missing operands treated as `0`. |
| `mod` | `a b -- (a%b)` | Modulo. Sign of result follows the dividend (`a`). | Modulo by zero: push `0`. Fewer than 2 elements: missing operands treated as `0`. |
| `eq` | `a b -- (a==b ? 1 : 0)` | Equality test | Fewer than 2 elements: missing operands treated as `0` |
| `gt` | `a b -- (a>b ? 1 : 0)` | Greater than (`a > b`) | Fewer than 2 elements: missing operands treated as `0` |
| `lt` | `a b -- (a<b ? 1 : 0)` | Less than (`a < b`) | Fewer than 2 elements: missing operands treated as `0` |
| `loop_start` | Conditional | Peek stack top: if nonzero, begin recording loop body. If zero, skip to matching `loop_end`. See §5. | Empty stack: treat as `0` (skip). Already at max nesting depth (256): no-op. |
| `loop_end` | Conditional | Pop stack top: if nonzero, replay recorded body. If zero, discard buffer and exit loop. See §5. | No active loop: no-op. Empty stack at check: treat as `0` (exit loop). |
| `print_num` | `a --` | Pop top and print its decimal string representation to the output stream | Empty stack: print `"0"` |
| `print_char` | `a --` | Pop top, compute `value % 256` as a byte, write to the output stream | Empty stack: no-op (do not print NUL) |
| `read` | `-- a` | Push one byte from the input stream onto the stack | No input available: push `0` |
| `store` | `val addr --` | Pop `addr` (top), pop `val` (second), store `val` at `memory[addr]` | Negative `addr`: no-op. Fewer than 2 elements: missing operands treated as `0` (store `0` at address `0`). |
| `load` | `addr -- val` | Pop `addr`, push `memory[addr]` onto the stack | Negative `addr`: push `0`. Uninitialized `addr`: push `0`. Empty stack: push `memory[0]`. |

**Total: 19 opcodes.**

### 3.4 Opcode Enumeration

The canonical opcode set, for use in event interface definitions and tuning files:

```
Opcode = dup | swap | drop | rotate
       | add | sub | mul | div | mod
       | eq | gt | lt
       | loop_start | loop_end
       | print_num | print_char | read
       | store | load
```

## 4. Event Interface

### 4.1 Overview

The VM is **event-driven**. It receives a stream of `Event` values and processes them sequentially. There is no instruction pointer, no program counter, no stored program. **The stream of events IS the program.**

The event interface is the **only** way to drive the VM. The VM does not know or care whether events came from a MIDI controller, a microphone, a test harness, or a file. All events are identical at the VM boundary.

### 4.2 Event Types

```
Event = Push(value: i64) | Op(opcode: Opcode)
```

- **`Push(value)`** pushes the given `i64` value onto the stack. This is how notes become values — the tuning file decides what value each note produces.
- **`Op(opcode)`** executes the named opcode per the instruction set table in §3.

`Push` is **not** an opcode. It is a distinct event type. The tuning file decides when to emit `Push` vs. `Op` events based on the performer's musical input.

### 4.3 Event Ordering

Events are processed **strictly in the order received**. The VM maintains no internal clock or timer. Event processing is synchronous — each event completes before the next begins (except during loop replay; see §5).

### 4.4 Optional Metadata

Each event **may** carry metadata for forward compatibility:

```
EventMetadata = {
  velocity: Option<u8>,      // 0–127, MIDI-style
  duration_ms: Option<u32>,  // Duration in milliseconds
}
```

**The v1 VM ignores metadata entirely.** Metadata exists for forward compatibility with v2 features (tempo-aware operations, velocity-based execution). Tuning files may use metadata in their own mapping logic, but the VM does not read it.

### 4.5 Source Independence

The VM processes events identically regardless of their origin. A conforming implementation must not vary behavior based on the event source. This guarantees that:

- Test harnesses can drive the VM with synthetic event sequences.
- Recorded performances can be replayed deterministically.
- Different input devices (MIDI, microphone, network) produce identical VM behavior for identical event sequences.

## 5. Control Flow — Loop Pedal Model

### 5.1 Overview

Haydn's only control flow mechanism is the **loop pedal** — a recording/replay system modeled after how musicians use loop pedal hardware. The performer triggers `loop_start` and `loop_end` operations; the VM records and conditionally replays the intervening events.

This design gives Haydn **while-loop semantics**, enabling a direct 1:1 mapping from Brainfuck's `[` and `]` brackets. See §7 for the Turing completeness proof.

### 5.2 Loop State Machine

The VM maintains a **loop stack** of event buffers and tracks one of three states:

```
LoopState = Normal | Recording(depth) | Replaying(depth, position)
```

- **`Normal`**: The VM processes events from the live input stream.
- **`Recording(depth)`**: The VM processes events from the live input stream AND records them into the current loop buffer. `depth` is the current nesting level (1 = outermost loop).
- **`Replaying(depth, position)`**: The VM processes events from a recorded buffer at the given position. `depth` is the nesting level.

### 5.3 State Transition Table

| Current State | Event | Action | Next State |
|---------------|-------|--------|------------|
| Normal | `Push(v)` | Push `v` onto stack | Normal |
| Normal | `Op(loop_start)` | **Peek** stack top. If zero or empty → skip to after matching `loop_end`. If nonzero → push new empty buffer onto loop stack. | Normal (if skipped) or Recording(1) |
| Normal | `Op(loop_end)` | No-op (unmatched `loop_end`) | Normal |
| Normal | `Op(other)` | Execute opcode | Normal |
| Recording(d) | `Push(v)` | Push `v` onto stack AND append `Push(v)` to current buffer | Recording(d) |
| Recording(d) | `Op(loop_start)` | **Peek** stack top. If zero or empty → append skip-marker, skip to matching `loop_end` within recording. If nonzero → push new buffer onto loop stack, append to parent buffer. | Recording(d) (if skipped) or Recording(d+1) |
| Recording(d) | `Op(loop_end)` at d=1 | **Pop** stack top. Nonzero → begin replaying current buffer from position 0. Zero → discard buffer, pop loop stack. | Replaying(1, 0) or Normal |
| Recording(d) | `Op(loop_end)` at d>1 | **Pop** stack top. Nonzero → begin replaying inner buffer. Zero → discard inner buffer, pop loop stack. | Replaying(d, 0) or Recording(d−1) |
| Recording(d) | `Op(other)` | Execute opcode AND append `Op(other)` to current buffer | Recording(d) |
| Replaying(d, pos) | buffer[pos] = `Push(v)` | Push `v` onto stack | Replaying(d, pos+1) |
| Replaying(d, pos) | buffer[pos] = `Op(loop_start)` | **Peek** stack top. If nonzero → push new buffer, begin recording nested loop. If zero → skip to matching `loop_end` in buffer. | Recording(d+1) or Replaying(d, pos after match) |
| Replaying(d, pos) | buffer[pos] = `Op(loop_end)` | **Pop** stack top. Nonzero → replay inner buffer. Zero → exit inner loop. | Replaying or reduce depth |
| Replaying(d, pos) | buffer[pos] = `Op(other)` | Execute opcode | Replaying(d, pos+1) |
| Replaying(d, end of buffer) | — | **Pop** stack top. Nonzero → reset pos to 0 (replay again). Zero → discard buffer, pop loop stack. | Replaying(d, 0) or Recording(d−1) or Normal |

### 5.4 Key Rules

1. **`loop_start` peeks, `loop_end` pops.** `loop_start` does not consume the stack top — it only inspects it to decide whether to enter the loop. `loop_end` pops (consumes) the stack top to decide whether to replay or exit.

2. **While-loop semantics.** If the stack top is zero (or the stack is empty) when `loop_start` is encountered, the entire loop body is skipped. This means a loop body may execute zero times — true while-loop behavior, not do-while.

3. **Execute during recording.** During Recording state, opcodes are **both executed and recorded**. The loop body runs on its first pass (the recording pass), then conditionally replays.

4. **Live input is queued during replay.** When the VM is in Replaying state, new live input events are **queued**, not interleaved. The VM processes the entire replay buffer (or exits the loop) before accepting new live events. This prevents chaotic interleaving of live and replayed events.

5. **Matching `loop_end` during skip.** When `loop_start` skips (stack top is zero), it must find the matching `loop_end` accounting for nesting depth — inner `loop_start`/`loop_end` pairs are matched and skipped over.

6. **Maximum nesting depth: 256.** A `loop_start` that would exceed depth 256 is a no-op. This prevents unbounded memory consumption from deeply nested loops.

7. **Unmatched boundaries.** `loop_end` in Normal state (no active loop) is a no-op. `loop_start` at max depth is a no-op. Neither condition is an error.

8. **Stream end during loop.** If the event stream ends while loops are open (Recording or Replaying state), all open loop buffers are discarded and the VM returns to Normal state. The "program" is the events received — if you stop playing inside a loop, all open loops close gracefully.

## 6. Silence and Timing

### 6.1 Silence

Silence is the **absence of events**. The VM is purely event-driven and has no internal clock. When no events arrive, no computation occurs. The VM simply waits.

The specification assigns no special semantics to silence. If the input layer or tuning file wishes to assign meaning to silence (e.g., emitting a specific operation after a timeout), that is the tuning file's responsibility — not the VM's.

### 6.2 Timing

The VM defines **no timing semantics**. Events are processed in the order of receipt with no concept of tempo, rhythm, or duration. The VM does not measure time between events. Two events arriving 1 millisecond apart or 10 seconds apart are processed identically.

### 6.3 Sustained Notes

The specification assigns no special behavior to note duration. If the input layer detects a sustained (held) note and the tuning file decides to generate events from it (e.g., repeating `dup` on sustain), the VM processes those events like any others. The tuning file decides what events, if any, to generate from sustained notes.

### 6.4 Deferred to v2

The following time-related features are explicitly deferred to a future version:

- **Tempo-aware operations** (operations whose behavior depends on BPM or event rate)
- **Dynamic-expression-based operations** (crescendo, decrescendo, ritardando as semantic elements)
- **Rhythm-pattern detection** (recognizing rhythmic patterns as compound operations)

The optional metadata on events (§4.4) is the v1 hook for forward compatibility with these features.

## 7. Turing Completeness

### 7.1 Overview

Haydn is Turing-complete. We prove this by showing that any Brainfuck program can be mechanically translated into a Haydn event sequence that produces identical I/O and halting behavior.

### 7.2 Brainfuck Reference

Brainfuck operates on a tape of integer cells (all initially `0`) with a movable data pointer (initially at cell `0`). It has 8 instructions:

| BF Instruction | Semantics |
|----------------|-----------|
| `>` | Move data pointer one cell right |
| `<` | Move data pointer one cell left |
| `+` | Increment the cell at the data pointer |
| `-` | Decrement the cell at the data pointer |
| `.` | Output the byte value of the cell at the data pointer |
| `,` | Read one byte of input into the cell at the data pointer |
| `[` | If the cell at the data pointer is `0`, jump forward past the matching `]` |
| `]` | If the cell at the data pointer is nonzero, jump back to the matching `[` |

### 7.3 Compilation Convention

The translation uses Haydn's addressable memory to simulate the BF tape:

- **Pointer storage:** Haydn `memory[0]` holds the current BF data pointer value.
- **Cell mapping:** BF cell `i` maps to Haydn `memory[i + 1]`. The offset of 1 reserves `memory[0]` for the pointer.
- **Stack convention:** The stack is empty between BF instruction boundaries. Inside loops, a check value from `loop_start`'s peek may sit at the bottom of the stack — it does not interfere with operations, which only touch the top elements.
- **Initialization:** The event sequence begins with `Push(1), Push(0), Op(store)`, which stores `1` at `memory[0]` — setting the data pointer to cell 0 (which is `memory[1]`).

### 7.4 Instruction Mapping

Each BF instruction compiles to a fixed Haydn event sequence. The `compile` function is total — every BF instruction has exactly one translation.

| BF | Haydn Event Sequence | Description |
|----|---------------------|-------------|
| `>` | `Push(0), Op(load), Push(1), Op(add), Push(0), Op(store)` | Load pointer, increment, store back |
| `<` | `Push(0), Op(load), Push(1), Op(sub), Push(0), Op(store)` | Load pointer, decrement, store back |
| `+` | `Push(0), Op(load), Op(dup), Op(load), Push(1), Op(add), Op(swap), Op(store)` | Load pointer, load cell, increment cell, store cell |
| `-` | `Push(0), Op(load), Op(dup), Op(load), Push(1), Op(sub), Op(swap), Op(store)` | Load pointer, load cell, decrement cell, store cell |
| `.` | `Push(0), Op(load), Op(load), Op(print_char)` | Load pointer, load cell, print as byte |
| `,` | `Push(0), Op(load), Op(read), Op(swap), Op(store)` | Load pointer, read input, store at cell |
| `[` | `Push(0), Op(load), Op(load), Op(loop_start)` | Load pointer, load cell, loop_start peeks cell value |
| `]` | `Push(0), Op(load), Op(load), Op(loop_end), Op(drop)` | Load pointer, load cell, loop_end pops cell value; drop cleans up `[`'s peek residue |

**Compilation function:** Given a BF program string, `compile(program)` = `Push(1), Push(0), Op(store)` (setup) followed by the concatenation of each BF character's mapping from the table above, in order. Non-instruction characters are ignored (BF convention).

### 7.5 Worked Example: BF `[-]`

The BF program `[-]` sets the current cell to zero. Below is a complete trace assuming cell 0 initially contains `3`.

**Setup:** `Push(1), Push(0), Op(store)` → `memory[0] = 1`, stack: `[]`

**Pre-populate cell 0 with 3** (via three `+` instructions, abbreviated): After three applications of the `+` mapping, `memory[1] = 3`, stack: `[]`.

**`[` mapping:** `Push(0), Op(load), Op(load), Op(loop_start)`

| Step | Event | Stack | Note |
|------|-------|-------|------|
| 1 | `Push(0)` | `[0]` | |
| 2 | `Op(load)` | `[1]` | Loads `memory[0]` = pointer = 1 |
| 3 | `Op(load)` | `[3]` | Loads `memory[1]` = cell value = 3 |
| 4 | `Op(loop_start)` | `[3]` | Peeks `3` ≠ 0 → enter Recording(1) |

**`-` mapping (recorded and executed):** `Push(0), Op(load), Op(dup), Op(load), Push(1), Op(sub), Op(swap), Op(store)`

| Step | Event | Stack | Note |
|------|-------|-------|------|
| 5 | `Push(0)` | `[3, 0]` | Recorded |
| 6 | `Op(load)` | `[3, 1]` | Loads pointer. Recorded |
| 7 | `Op(dup)` | `[3, 1, 1]` | Recorded |
| 8 | `Op(load)` | `[3, 1, 3]` | Loads cell value. Recorded |
| 9 | `Push(1)` | `[3, 1, 3, 1]` | Recorded |
| 10 | `Op(sub)` | `[3, 1, 2]` | 3 − 1 = 2. Recorded |
| 11 | `Op(swap)` | `[3, 2, 1]` | Recorded |
| 12 | `Op(store)` | `[3]` | Stores 2 at `memory[1]`. Recorded |

**`]` mapping (recorded except `loop_end` and `drop`):**

| Step | Event | Stack | Note |
|------|-------|-------|------|
| 13 | `Push(0)` | `[3, 0]` | Recorded |
| 14 | `Op(load)` | `[3, 1]` | Loads pointer. Recorded |
| 15 | `Op(load)` | `[3, 2]` | Loads cell value (now 2). Recorded |
| 16 | `Op(loop_end)` | `[3]` | Pops `2` ≠ 0 → Replaying(1, 0). NOT recorded |
| 17 | `Op(drop)` | — | Queued (live input queued during replay) |

**Buffer contents:** `[Push(0), Op(load), Op(dup), Op(load), Push(1), Op(sub), Op(swap), Op(store), Push(0), Op(load), Op(load)]`

**Replay iteration 1:** Stack starts at `[3]`. Body decrements `memory[1]` from 2 to 1. End-of-buffer pops `1` (the cell value loaded by the last two events in the buffer) — nonzero → replay again. Stack: `[3]`.

**Replay iteration 2:** Body decrements `memory[1]` from 1 to 0. End-of-buffer pops `0` → exit loop. Stack: `[3]`. State returns to Normal.

**Queued `Op(drop)` executes:** Stack `[3]` → `[]`.

**Result:** `memory[1] = 0`. Cell zeroed. Stack empty. ✓

### 7.6 Loop Mechanics in the Reduction

**Skip path (`[` when cell is zero):** `loop_start` peeks `0` and skips to after the matching `loop_end`, scanning the event stream and accounting for nested `loop_start`/`loop_end` pairs. The `Op(drop)` that follows `loop_end` in the `]` mapping is the first event after the skip — it drops the stale `0` from `[`'s peek, restoring the stack to empty.

**Nested loops:** Inner `[`/`]` pairs create nested Recording/Replaying states via the loop stack. Each level's `Op(drop)` cleans up its own peek residue. The BF nesting structure maps directly to Haydn's loop nesting.

### 7.7 Proof Argument

**Totality.** All 8 BF instructions have a defined, fixed-length mapping in the table above. The `compile` function is total.

**I/O Preservation.** BF `.` maps to `Op(print_char)` applied to the cell value — identical byte output. BF `,` maps to `Op(read)` — identical byte input. Haydn's `read` returns `0` when no input is available, consistent with the standard BF "0 on EOF" convention.

**Halting Preservation.** BF `[`/`]` maps to `loop_start`/`loop_end` with while-loop semantics (§5). Entry condition: cell at pointer is zero → skip (equivalent to BF `[` skipping when cell is 0). Continuation condition: cell at pointer is nonzero → replay (equivalent to BF `]` jumping back when cell is nonzero). Loop termination occurs under identical conditions in both languages.

**Value Range.** BF conventionally uses byte-sized cells (0–255). Haydn uses `i64`. The reduction is valid because: (a) the standard BF Turing completeness proof uses unbounded integer cells, (b) Haydn's `i64` range (−2⁶³ to 2⁶³−1) is a strict superset of any practical BF cell range, and (c) `print_char` applies `% 256`, preserving BF's byte output semantics.

**Conclusion.** Since Brainfuck is Turing-complete and every Brainfuck program mechanically compiles to a Haydn event sequence with identical I/O and halting behavior, **Haydn is Turing-complete**. ∎

## 8. Examples

The following examples show Haydn event sequences with step-by-step stack traces. These are raw VM-level programs — in practice, a performer would never construct these event sequences manually; they would emerge from playing a musical instrument through a tuning file.

### 8.1 Hello — Push and Print

Print the string "Hello" by pushing ASCII values and printing each as a character.

**Event sequence:**

```
Push(72), Op(print_char),
Push(101), Op(print_char),
Push(108), Op(print_char),
Push(108), Op(print_char),
Push(111), Op(print_char)
```

**Trace:**

| Step | Event | Stack | Output |
|------|-------|-------|--------|
| 1 | `Push(72)` | `[72]` | |
| 2 | `Op(print_char)` | `[]` | `H` (72 % 256 = 72 = 'H') |
| 3 | `Push(101)` | `[101]` | |
| 4 | `Op(print_char)` | `[]` | `e` |
| 5 | `Push(108)` | `[108]` | |
| 6 | `Op(print_char)` | `[]` | `l` |
| 7 | `Push(108)` | `[108]` | |
| 8 | `Op(print_char)` | `[]` | `l` |
| 9 | `Push(111)` | `[111]` | |
| 10 | `Op(print_char)` | `[]` | `o` |

**Output:** `Hello`

### 8.2 Arithmetic — Add Two Numbers

Push 3 and 4, add them, print the result as a number.

**Event sequence:**

```
Push(3), Push(4), Op(add), Op(print_num)
```

**Trace:**

| Step | Event | Stack | Output |
|------|-------|-------|--------|
| 1 | `Push(3)` | `[3]` | |
| 2 | `Push(4)` | `[3, 4]` | |
| 3 | `Op(add)` | `[7]` | |
| 4 | `Op(print_num)` | `[]` | `7` |

**Output:** `7`

### 8.3 Countdown Loop

Count down from 5 to 1, printing each number on a new line, using the loop pedal mechanism.

**Event sequence:**

```
Push(5),
Op(loop_start),
  Op(dup), Op(print_num),
  Push(10), Op(print_char),
  Push(1), Op(sub),
  Op(dup),
Op(loop_end)
```

**Trace (first iteration — recording pass):**

| Step | Event | Stack | Output | Recorded? |
|------|-------|-------|--------|-----------|
| 1 | `Push(5)` | `[5]` | | No |
| 2 | `Op(loop_start)` | `[5]` | | No — peeks `5` ≠ 0, enters Recording(1) |
| 3 | `Op(dup)` | `[5, 5]` | | Yes |
| 4 | `Op(print_num)` | `[5]` | `5` | Yes |
| 5 | `Push(10)` | `[5, 10]` | | Yes |
| 6 | `Op(print_char)` | `[5]` | `\n` | Yes |
| 7 | `Push(1)` | `[5, 1]` | | Yes |
| 8 | `Op(sub)` | `[4]` | | Yes |
| 9 | `Op(dup)` | `[4, 4]` | | Yes |
| 10 | `Op(loop_end)` | `[4]` | | No — pops `4` ≠ 0 → Replaying(1, 0) |

**Buffer:** `[Op(dup), Op(print_num), Push(10), Op(print_char), Push(1), Op(sub), Op(dup)]`

**Replay iterations:**

| Iteration | Stack at start | Prints | Cell value after sub | End-of-buffer check |
|-----------|---------------|--------|---------------------|-------------------|
| 2 | `[4]` | `4\n` | 3 | Pop `3` ≠ 0 → replay |
| 3 | `[3]` | `3\n` | 2 | Pop `2` ≠ 0 → replay |
| 4 | `[2]` | `2\n` | 1 | Pop `1` ≠ 0 → replay |
| 5 | `[1]` | `1\n` | 0 | Pop `0` = 0 → exit loop |

**Final stack:** `[0]`

**Output:**
```
5
4
3
2
1
```

## 9. Specification Summary

| Property | Value |
|----------|-------|
| **Opcodes** | 19 (`dup`, `swap`, `drop`, `rotate`, `add`, `sub`, `mul`, `div`, `mod`, `eq`, `gt`, `lt`, `loop_start`, `loop_end`, `print_num`, `print_char`, `read`, `store`, `load`) |
| **Event types** | 2: `Push(value: i64)`, `Op(opcode: Opcode)` |
| **Value type** | `i64` — signed 64-bit integer, wrapping arithmetic |
| **Stack** | Unbounded LIFO, initially empty |
| **Memory** | Addressable `i64` array, indexed by non-negative `i64`, initially all zeros |
| **Control flow** | Loop-based only — no jumps, no goto, no instruction pointer |
| **Turing complete** | Yes — via Brainfuck reduction (§7) |
| **Error handling** | All edge cases produce defined, deterministic results. The VM never panics. |
| **Timing** | None — purely event-driven, no internal clock |
| **Input model** | Event stream of `Push` and `Op` events, source-independent |

### Design Principles

1. **The stream of events IS the program.** There is no stored program, no instruction pointer, no program counter.
2. **The VM is source-independent.** MIDI, microphone, test harness, or file — all produce identical behavior for identical event sequences.
3. **The VM never crashes.** Every edge case (empty stack, division by zero, invalid address, unmatched loop boundaries) has a defined result. The performer's experience is continuous.
4. **Music is art.** The VM is a fixed computation engine; the musical mapping is configurable via tuning files. Different instruments and styles produce different programs from the same gestures.

---

*Haydn Language Specification v1.0 — 2026*
