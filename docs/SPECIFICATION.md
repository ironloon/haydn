# Haydn Language Specification

**Version:** 1.0
**Status:** Draft

> Haydn is an esoteric programming language where live musical performance IS the program.

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
