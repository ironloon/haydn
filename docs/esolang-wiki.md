<!-- Working paper. Not yet submitted. -->

# Haydn

**Haydn** is an esoteric programming language in which programs are performed live on musical instruments. There is no source code file — the performer's real-time musical input, via MIDI controller or microphone, constitutes the program. A stack-based virtual machine with 19 opcodes interprets events as they arrive, producing output during the performance itself.

Designed by Jared, 2026. Implemented in Rust.

## Overview

Haydn's defining characteristic is the absence of a textual source representation. The performer plays notes on an instrument; a tuning file maps those notes to computational events; the VM executes them. Two event types drive the machine:

- **Push(value)** — Place a signed 64-bit integer on the stack.
- **Op(opcode)** — Execute one of 19 opcodes.

Events are source-independent. The VM processes the same event identically whether it originated from a MIDI keyboard, a singing voice, or a test harness. The musical mapping is a separate concern, configured via TOML tuning files. Different instruments produce different programs from the same gestures.

## Computational Model

- **Stack:** Unbounded LIFO of signed 64-bit integers (`i64`). Initially empty.
- **Memory:** Addressable array of `i64` values indexed by non-negative integers. Initially all zero.
- **I/O:** Byte streams. Output via `print_num` and `print_char`; input via `read`. Read is non-blocking — returns 0 when no input is available.
- **Control flow:** Loop pedal model (see below).
- **Arithmetic:** Wrapping (modular) overflow. No traps.
- **Edge cases:** The VM never panics and never halts on error. Every edge case has defined behavior. Missing stack operands are treated as 0.

## Instruction Set

Haydn has 19 opcodes. Stack effects use Forth notation: `before -- after` (rightmost value is top of stack).

| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `dup` | `a -- a a` | Duplicate top of stack |
| `swap` | `a b -- b a` | Swap top two elements |
| `drop` | `a --` | Remove top of stack |
| `rotate` | `a b c -- b c a` | Rotate top three: third moves to top |
| `add` | `a b -- (a+b)` | Addition |
| `sub` | `a b -- (a-b)` | Subtraction. `b` is top, `a` is second. |
| `mul` | `a b -- (a*b)` | Multiplication |
| `div` | `a b -- (a/b)` | Integer division, truncated toward zero |
| `mod` | `a b -- (a%b)` | Modulo |
| `eq` | `a b -- (a==b ? 1 : 0)` | Equality test |
| `gt` | `a b -- (a>b ? 1 : 0)` | Greater than |
| `lt` | `a b -- (a<b ? 1 : 0)` | Less than |
| `loop_start` | *(conditional)* | Peek top: nonzero → record; zero → skip to `loop_end` |
| `loop_end` | *(conditional)* | Pop top: nonzero → replay buffer; zero → exit loop |
| `print_num` | `a --` | Print decimal representation |
| `print_char` | `a --` | Print `value % 256` as byte |
| `read` | `-- a` | Push one byte from input (0 if unavailable) |
| `store` | `val addr --` | Store `val` at `memory[addr]` |
| `load` | `addr -- val` | Load `memory[addr]` onto stack |

## Event Interface

The VM accepts a stream of events, each either `Push(value: i64)` or `Op(opcode)`. Events arrive one at a time and are executed immediately. The interface is the same regardless of input source.

```
enum Event {
    Push(i64),
    Op(Opcode),
}
```

## Control Flow

Haydn uses a **loop pedal** model for control flow, inspired by guitar loop pedals:

- **`loop_start`** peeks at the top of stack (does not pop). If the value is nonzero, the VM begins recording all subsequent events into a buffer. If zero, the VM skips forward to the matching `loop_end`.
- **`loop_end`** pops the top of stack. If the value is nonzero, the recorded buffer replays from the beginning. If zero, the buffer is discarded and the loop exits.

This provides standard while-loop semantics. Loops nest to a depth of 256. The buffer contains the events between `loop_start` and `loop_end`, replayed as many times as the stack demands.

The "execute-during-record" property means events are both executed and recorded on the first pass. Subsequent iterations replay from the buffer.

## Hello World

Using the default piano tuning, which maps low notes (C2–B3) to `Push(midi_note − 60)` and high notes (C4+, white keys) to opcodes:

**Event sequence** (for the letter 'H', ASCII 72):

```
Push(-8), Push(-9), Op(Mul), Op(PrintChar)
```

(−8) × (−9) = 72. `PrintChar` pops 72 and writes 'H'.

**Full program in piano note names:**

```
H  (72):  E3  D#3  G4  A5
e (101):  D3  D3   G4  B3  D4  A5
l (108):  C3  D#3  G4  A5
l (108):  C3  D#3  G4  A5
o (111):  C3  D3   G4  D#3 C4  A5
, ( 44):  G#3 C#3  G4  A5
  ( 32):  G#3 E3   G4  A5
W ( 87):  D#3 D3   G4  A3  C4  A5
o (111):  C3  D3   G4  D#3 C4  A5
r (114):  F#3 F2   G4  A5
l (108):  C3  D#3  G4  A5
d (100):  D3  D3   G4  A5
! ( 33):  A3  C#3  G4  A5
\n( 10):  A#3 G3   G4  A5
```

56 events. Each character is built by multiplying two value-zone numbers to produce the target ASCII code, then printing with A5 (`print_char`). The low–high register alternation — left hand pushes values, right hand triggers operations — is the characteristic Haydn sound.

## Computational Class

Haydn is **Turing-complete**, proven by reduction from Brainfuck.

The reduction maps BF's tape to Haydn's addressable memory (memory cell 0 stores the data pointer; BF cells occupy addresses 1, 2, 3, ...). Each BF instruction compiles to a fixed sequence of 4–8 Haydn events:

| BF | Haydn |
|----|-------|
| `>` | `Push(0) Load Push(1) Add Push(0) Store` |
| `<` | `Push(0) Load Push(1) Sub Push(0) Store` |
| `+` | `Push(0) Load Dup Load Push(1) Add Swap Store` |
| `-` | `Push(0) Load Dup Load Push(1) Sub Swap Store` |
| `.` | `Push(0) Load Load PrintChar` |
| `,` | `Push(0) Load Read Swap Store` |
| `[` | `Push(0) Load Load LoopStart` |
| `]` | `Push(0) Load Load LoopEnd Drop` |

The canonical BF Hello World compiles to approximately 800 Haydn events and produces identical output when executed by the VM.

The full proof — including the data pointer invariant and correctness argument for each instruction mapping — is in the [language specification, §7](docs/SPECIFICATION.md).

## Tuning System

Tuning files are TOML configuration files that map MIDI notes (or detected audio pitches) to VM events. Three default tunings are provided:

- **Piano** — Value zone C2–B3, operations on white keys C4+. The general-purpose default.
- **Voice** — Maps detected pitch ranges to events. Wider tolerance for intonation imprecision.
- **Guitar** — Maps fret positions via MIDI guitar interface.

Users can create custom tunings to map any instrument's gestures to any combination of pushes and operations. The tuning file format is documented separately.

## External Resources

- **Interpreter:** `cargo install haydn` (Rust, MIT license)
- **Source code:** [GitHub](https://github.com/jaredmontoya/haydn)
- **Language specification:** [SPECIFICATION.md](docs/SPECIFICATION.md)
- **Tuning format:** [TUNING.md](docs/TUNING.md)
- **Examples:** [Hello World](examples/hello-world.md), [Cat](examples/cat.md), [Truth Machine](examples/truth-machine.md), [FizzBuzz](examples/fizzbuzz.md)
