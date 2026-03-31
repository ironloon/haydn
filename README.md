# Haydn

*A programming language in which programs are performed, not written.*

```
cargo install haydn
```

## Abstract

We present Haydn, a novel programming language in which programs are performed rather than written. The performer's live musical input — via MIDI controller or microphone — constitutes the source code, interpreted by a stack-based virtual machine in real time. The language achieves Turing completeness via reduction from Brainfuck (§7 of the specification) and provides example programs performable on standard musical instruments. We argue that this approach to computation is no less rigorous than conventional programming, merely less convenient, and considerably more likely to receive a standing ovation.

## 1. Introduction

Haydn belongs to the tradition of esoteric programming languages — Brainfuck, Piet, Befunge, and their descendants — languages that are deliberately unusual but computationally serious. What distinguishes Haydn from its predecessors is the absence of source code. There are no files. There is no editor. The performer sits at an instrument and plays. The notes they produce are captured, mapped to computational events, and executed by a virtual machine, all in real time. The output appears on screen as the music unfolds.

The system consists of two layers:

1. **The Virtual Machine.** A stack-based computation engine receiving a stream of `Push(value)` and `Op(opcode)` events. Nineteen opcodes handle arithmetic, stack manipulation, I/O, memory access, and control flow. The VM is entirely deterministic and never panics — because the performer's experience must be continuous.

2. **Tuning Files.** TOML configuration files that map musical gestures to VM events. A pianist, vocalist, and guitarist each use a different tuning, but they all drive the same machine. The tuning is the interpretation; the VM is the law.

The name references Joseph Haydn, the composer known for structural wit and the strategic deployment of surprise. Readers familiar with the Surprise Symphony will appreciate that the first Hello World program was also premiered to notable effect.

## 2. Quick Start

Install from crates.io:

```
cargo install haydn
```

Launch with MIDI input:

```
haydn --input midi
```

Or with microphone input, for those who prefer to sing their programs:

```
haydn --input audio
```

### Hello World on Piano

Using the default piano tuning, the program `Hello, World!\n` is performed as follows. Low notes (C2–B3) push values onto the stack; high notes (C4 and above) trigger operations. Each character is built by multiplying two pushed values and printing the result:

```
H  (72):  E3  D#3  G4  A5          (-8) × (-9) = 72
e (101):  D3  D3   G4  B3  D4  A5  (-10)² - (-1) = 101
l (108):  C3  D#3  G4  A5          (-12) × (-9) = 108
l (108):  C3  D#3  G4  A5          (-12) × (-9) = 108
o (111):  C3  D3   G4  D#3 C4  A5  (-12) × (-10) + (-9) = 111
```

The pattern continues for `, World!\n`. Each A5 (print_char) pops the computed ASCII value from the stack and writes the character to the output stream. The full program — 56 events, roughly 15 seconds at a comfortable tempo — is documented in [examples/hello-world.md](examples/hello-world.md).

## 3. Methods

### Event Model

The VM accepts exactly two event types:

- **`Push(value)`** — Place a signed 64-bit integer on the stack.
- **`Op(opcode)`** — Execute one of 19 opcodes.

Events are source-independent. The VM does not know whether they originated from a MIDI controller, a microphone, or a test harness. This is by design.

### The Stack Machine

Computation proceeds by the conventional stack discipline: push operands, apply operators, consume results. All values are signed 64-bit integers with wrapping arithmetic. Memory is an addressable array of integers indexed by non-negative addresses. The stack has no maximum depth. The VM never panics.

The 19 opcodes fall into five categories:

| Category | Opcodes |
|----------|---------|
| Stack | `dup`, `swap`, `drop`, `rotate` |
| Arithmetic | `add`, `sub`, `mul`, `div`, `mod` |
| Comparison | `eq`, `gt`, `lt` |
| Control | `loop_start`, `loop_end` |
| I/O & Memory | `print_num`, `print_char`, `read`, `store`, `load` |

The complete opcode table with stack effects and edge case behavior is in [the specification, §3.3](docs/SPECIFICATION.md).

### Control Flow: The Loop Pedal

Haydn's control flow borrows from the guitar loop pedal. `loop_start` peeks at the top of stack: if nonzero, it begins recording all subsequent events into a buffer; if zero, execution skips to the matching `loop_end`. When `loop_end` is reached, it pops the stack: if nonzero, the recorded buffer replays from the beginning; if zero, the loop exits. This gives standard while-loop semantics — but the metaphor is musical. You press the pedal, play a phrase, and it repeats until you tell it to stop.

Loops nest to a depth of 256. The buffer records the events between `loop_start` and `loop_end`, replaying them as many times as the stack demands.

### Tuning System

A tuning file maps musical input to VM events. The default piano tuning divides the keyboard:

- **Value zone** (MIDI 36–59, C2–B3): Each note pushes `midi_note − 60`, yielding values from −24 to −1.
- **Operation zone** (MIDI 60+, white keys in C major): Each white key triggers an opcode — C4 for `add`, D4 for `sub`, G4 for `mul`, and so on.

Different instruments produce different programs from the same gestures. A vocalist's tuning maps detected pitches; a guitarist's maps fret positions. The tuning is the lens through which the instrument sees the machine.

Custom tunings are TOML files. The format is documented in [docs/TUNING.md](docs/TUNING.md).

## 4. Results

Four example programs demonstrate Haydn's capabilities in order of increasing complexity. All use the default piano tuning.

### Hello World — [examples/hello-world.md](examples/hello-world.md)

*Teaches: push, multiply, print.*

Builds each ASCII character from arithmetic on pushed values. Includes a complete tabular walkthrough tracing note → event → stack state → output for the first three characters.

### Cat — [examples/cat.md](examples/cat.md)

*Teaches: input/output.*

Reads bytes and echoes them. Eight notes. The shortest meaningful Haydn program.

### Truth Machine — [examples/truth-machine.md](examples/truth-machine.md)

*Teaches: conditional loops.*

Reads one character: if '0', prints '0' and halts; if '1', prints '1' forever. The loop pedal model handles both cases without branching instructions — the stack value determines whether the loop engages.

### FizzBuzz — [examples/fizzbuzz.md](examples/fizzbuzz.md)

*Teaches: loops, modular arithmetic, Turing completeness in practice.*

Constructed via the Brainfuck reduction (spec §7). Each BF instruction compiles mechanically to 4–8 Haydn events, producing a program too long to notate by hand but executable just the same. The capstone proof that anything computable is performable.

## 5. Discussion

### Music Is Art

The tuning system is Haydn's most distinctive feature. Because the mapping from music to computation is configurable, the same musical phrase produces different programs on different instruments. A C major scale on piano is one sequence of pushes and operations; the same pitches sung by a soprano, with their natural vibrato and imprecision, produce something else entirely. Your voice is unique, so is your program.

This is not a limitation. Tuning files are shareable artifacts — a tuning paired with a performance description becomes a seed for exploration. "Load this tuning and play ascending thirds" is a program specification that two performers will realize differently, producing programs that diverge in ways neither predicted. Whether this constitutes a feature or a deficiency depends on one's tolerance for the aleatory.

### Input Paths

Haydn supports two input methods:

- **MIDI** — Clean, discrete note events. Every key press maps unambiguously to a VM event. Ideal for precise programming.
- **Microphone** — Continuous audio analyzed for pitch and onset. Noisier, more expressive, and dependent on the performer's skill and the room's acoustics. The pitch detection algorithm (McLeod autocorrelation) tolerates reasonable imprecision, but the performer who sings slightly flat will discover that music theory and integer arithmetic do not always agree.

### On Practicality

Haydn is not a practical programming language. It is a serious one. The distinction matters. The VM is fully specified, deterministic, and Turing-complete. The test suite verifies every opcode, every edge case, and every example program. The impracticality is in the interface, not the computation — and the interface is the point.

## References

- [Language Specification](docs/SPECIFICATION.md) — Complete VM specification: opcodes, event interface, loop pedal model, Turing completeness proof
- [Tuning File Format](docs/TUNING.md) — How tuning files map musical input to VM events
- [Hello World](examples/hello-world.md) — Full example with tabular walkthrough
- [Cat](examples/cat.md) — I/O echo program
- [Truth Machine](examples/truth-machine.md) — Conditional loop example
- [FizzBuzz](examples/fizzbuzz.md) — Turing completeness via BF reduction
- [Esolang Wiki Draft](docs/esolang-wiki.md) — Community reference page (working paper)

## Acknowledgments

The authors gratefully acknowledge the assistance of large language models during the development of this work. Their contributions to implementation, documentation, and the navigating of particularly opaque compiler diagnostics were, on the whole, adequate.

## License

MIT
