# Haydn

## What This Is

Haydn is an esoteric programming language performed by singing or playing a musical instrument, interpreted and executed in real time. The performer's audio (via microphone) or MIDI input becomes code — notes push values, intervals trigger operations, and musical gestures drive a stack-based virtual machine. Built in Rust, distributed as a crate and standalone binary.

## Core Value

The live performance IS the program. Music becomes computation in real time, with immediate visual feedback showing the machine state as the performer plays.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Real-time MIDI input captures note events and maps to language operations
- [ ] Real-time microphone input analyzes pitch/onset and maps to language operations
- [ ] Stack-based virtual machine executes Haydn instructions (Turing-complete)
- [ ] Configurable "tuning files" map musical gestures to VM opcodes (different instruments/styles)
- [ ] Instruction set: push, add, sub, mul, div, mod, dup, swap, drop, rotate, eq, gt, lt, jump, conditional, loop, print num, print char, read, store, load
- [ ] CLI visualization displays stack state, current operation, and output during performance
- [ ] Continuous real-time interpretation — every musical event triggers immediate execution
- [ ] Full language specification document
- [ ] Fun, inspiring README in the tradition of great esolangs (esolang wiki style)
- [ ] Example programs: Hello World, FizzBuzz, and other classic demos performed musically
- [ ] Default tuning files for common instruments (keyboard, voice, guitar)
- [ ] `cargo install haydn` distribution

### Out of Scope

- Web playground / WASM build — future milestone, not v1
- GUI / rich TUI visualization — CLI only for v1
- Audio output / sonification — input only for v1, output is text/visual
- Multi-performer / ensemble mode — single performer for v1
- Recording/playback of performances — live only for v1

## Context

- Esoteric language tradition: Brainfuck, Piet, Befunge, Chicken, Shakespeare — languages that are deliberately unusual but computationally serious
- The name "Haydn" references Joseph Haydn, the composer known for wit, humor, and structural innovation (the "Surprise" Symphony, the "Farewell" Symphony)
- Stack-based computation chosen because it mirrors musical tension/resolution — notes accumulate (push), operations resolve (pop and compute)
- Layered architecture: the computation engine is fixed and well-specified, but the musical mapping is configurable via "tuning files" — this means a pianist, vocalist, guitarist, or drummer can all program in Haydn using gestures natural to their instrument
- Real-time audio analysis requires pitch detection (e.g., YIN algorithm or similar) for microphone input
- MIDI input is discrete and clean — easier to map; microphone is continuous and noisy — harder but more expressive
- Target audience: esolang enthusiasts, creative coders, music technologists, people who want to blow minds at work

## Constraints

- **Language**: Rust — performance, safety, and ecosystem for audio processing
- **Input**: Must support both MIDI and microphone as first-class input sources
- **Computation**: Must be Turing-complete (provably, with examples)
- **Real-time**: Latency must be low enough that performers perceive immediate response
- **Distribution**: Must be installable via `cargo install`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Stack-based VM | Mirrors musical tension/resolution; proven model for esolangs; easy to visualize | — Pending |
| Configurable tuning files | Don't restrict the artist — different instruments/styles map differently to opcodes | — Pending |
| Both MIDI + microphone input | MIDI is clean/discrete, mic is expressive/accessible — supporting both broadens who can perform | — Pending |
| CLI visualization (not GUI) | Keep v1 focused; CLI is sufficient for showing stack state and output | — Pending |
| Rust | Low-latency audio processing, memory safety, cargo ecosystem | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-28 after initialization*
