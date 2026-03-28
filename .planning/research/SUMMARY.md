# Project Research Summary

**Project:** Haydn
**Domain:** Real-time musical esoteric programming language
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

Haydn is a stack-based esoteric programming language where live musical performance IS the source code — notes push values, intervals trigger operations, and a configurable tuning file decouples musical gesture from computation. The Rust ecosystem is exceptionally well-suited for this: `cpal` (audio I/O), `midir` (MIDI), `pitch-detection` (McLeod algorithm), and `ratatui` (TUI) are all mature, pure-Rust, and actively maintained. The recommended architecture is a three-thread model (audio callback → lock-free ring buffer → processing thread → channel → UI thread) that respects hard real-time audio constraints while keeping the VM and visualization responsive.

The critical technical challenge is the microphone-to-opcode pipeline: pitch detection, onset detection, noise gating, and a musical gesture state machine must work together to produce clean, discrete note events from continuous, noisy audio. MIDI input is straightforward by comparison and should be the first end-to-end path. The hardest **design** challenge is jump/loop semantics in a live-input language — there is no static program to jump into, so the language spec must resolve this (buffer model, structured loops, or label model) before implementation begins.

The project sits at the intersection of two communities — esolang enthusiasts (who demand precise specs, Turing completeness proofs, and clever constraints) and music technologists (who demand low latency, MIDI integration, and performability). The "Hello World played on a piano" is the artifact that bridges both worlds and should be treated as the primary demo target. Research confidence is HIGH across all areas: the stack is well-documented, the architecture follows established real-time audio patterns, and the pitfalls are well-known from decades of audio programming and language implementation.

## Key Findings

### Recommended Stack

| Technology | Purpose | Confidence |
|------------|---------|------------|
| **Rust** (1.85+) | Core language — non-negotiable per project spec | HIGH |
| **cpal** 0.17.3 | Audio I/O (microphone capture), cross-platform, pure Rust | HIGH |
| **midir** 0.10.3 | MIDI input capture, cross-platform, callback-based | HIGH |
| **midly** 0.5.3 | MIDI message parsing (zero-copy, pairs with midir) | HIGH |
| **pitch-detection** 0.3.0 | McLeod pitch detector for monophonic real-time input | MEDIUM (stable but unmaintained) |
| **rustfft** 6.4.1 | SIMD-accelerated FFT (if custom pitch detection needed) | HIGH |
| **ringbuf** 0.4.8 | Lock-free SPSC ring buffer for audio thread communication | HIGH |
| **ratatui** 0.30.0 | Terminal UI for stack/output visualization | HIGH |
| **crossterm** 0.29.0 | Terminal backend for ratatui | HIGH |
| **serde** + **toml** | Tuning file deserialization (human-readable TOML format) | HIGH |
| **clap** 4.6.0 | CLI argument parsing | HIGH |
| **thiserror** + **anyhow** | Error handling (library + application layers) | HIGH |
| **tracing** | Structured diagnostics for debugging real-time issues | HIGH |

**Critical constraint:** No `tokio`/async runtime. Real-time audio requires OS threads + lock-free channels. No `Mutex` touching the audio thread. No `aubio` (GPL-incompatible with MIT).

### Expected Features

**Table Stakes (P0 — must ship):**
- Language specification (unambiguous, covering all edge cases — learn from Brainfuck's ambiguity mistakes)
- Stack-based VM with full instruction set (20+ opcodes)
- MIDI input with note-to-instruction mapping
- Configurable tuning file format (TOML) + default piano tuning
- CLI visualization (stack state, current operation, output)
- Hello World example program
- Turing completeness proof (formal BF reduction)
- `cargo install haydn` distribution

**Differentiators (P1 — should ship in v1.x):**
- Microphone input with real-time pitch detection (the "anyone with a voice can program" path)
- Dual input mode (MIDI + mic as first-class sources)
- Interval-based operation mapping (notes=data, intervals=operations — uniquely musical)
- Instrument-specific default tunings (voice, guitar)
- Additional example programs (FizzBuzz, Cat, Truth Machine)

**Anti-features (deliberately NOT building in v1):**
- Audio output/sonification — Haydn is not a worse Sonic Pi
- Web playground/WASM — major scope, defer to v2
- Chord recognition — polyphonic detection from mic is unsolved in real-time
- Tempo/rhythm awareness — respond to WHAT is played, not HOW FAST
- Recording/playback, ensemble mode, IDE support

### Architecture Approach

**Three-thread model:**
1. **Audio thread** (cpal callback) — writes raw samples to lock-free ring buffer. MUST NOT allocate, lock, or block.
2. **Processing thread** — reads samples, runs pitch/onset detection, receives MIDI events, maps through tuning engine, executes VM, publishes state snapshots.
3. **UI thread** (main) — ratatui render loop at ~30fps, receives snapshots via channel.

**Key architectural patterns:**
- Lock-free ring buffer (ringbuf/rtrb) between audio callback and processing thread
- Unified `MusicalEvent` enum with confidence field that normalizes both MIDI and mic input
- Tuning engine as a decoupling layer: `MusicalEvent → Instruction` via HashMap lookup
- Instruction budget per tick (1000 ops) to contain infinite loops
- Musical gesture state machine (SILENCE → NOTE_ACTIVE → TRANSITION) for mic input
- Interval detection window (~200-500ms) to distinguish intervals from independent note pushes

**Project structure:** 6 modules — `vm/`, `tuning/`, `events/`, `audio/`, `midi/`, `ui/` — clean separation of concerns.

### Critical Pitfalls

| # | Pitfall | Severity | Prevention |
|---|---------|----------|------------|
| 1 | **Allocating in audio callback** | CRITICAL | Pre-allocate all buffers. Audio callback only writes to ring buffer. Audit every line for hidden allocations (`clone()`, `collect()`, `format!()`). |
| 2 | **Mutex between audio and main thread** | CRITICAL | Lock-free communication only. Design data flow as one-directional message passing from day 1. Retrofitting is a near-rewrite. |
| 3 | **Pitch detection octave errors** | CRITICAL | Use pYIN or filtered McLeod. Clamp to instrument range via tuning file. Add hysteresis. Consider pitch-class mapping to make octave errors non-catastrophic. |
| 4 | **No onset detection** | CRITICAL | Implement onset detection BEFORE pitch detection. Without it, VM fires ops at audio-frame rate (~100/sec) instead of note rate (~1-5/sec). |
| 5 | **Jump semantics in live-input language** | HIGH | Resolve in language spec BEFORE implementation. Recommend structured loops (repeat-N) or label model over arbitrary address jumps. |
| 6 | **Stack overflow/underflow crashes** | HIGH | Hard stack limit (1024). Underflow returns 0, not panic. Never `unwrap()` in VM execution path. |
| 7 | **Latency accumulation** | HIGH | Budget ≤30ms total. Measure end-to-end, not per-component. Decouple render framerate from event rate. |
| 8 | **Noise floor triggering false events** | HIGH | Energy gate with configurable threshold. Calibration step at startup. Confidence threshold from pitch detector. |

## Implications for Roadmap

### Phase 1: Language Specification & VM Core
**Rationale:** Zero external dependencies. Defines the `Instruction` type that everything else depends on. All edge cases (division by zero, stack overflow/underflow, jump semantics) must be specified before implementation. The Turing completeness proof validates the instruction set design.
**Delivers:** Complete language spec document, working VM with all 20+ opcodes, comprehensive unit tests, formal TC proof.
**Features:** Stack operations, arithmetic, comparison/branching, I/O, memory, edge case handling.
**Pitfalls to address:** Stack overflow/underflow (#6), infinite loops (#8), jump semantics (#5/17), Turing completeness proof (#16).

### Phase 2: Tuning File System
**Rationale:** Depends only on the `Instruction` enum from Phase 1. The tuning file format is a load-bearing design decision — too simple and the language can't express real musical mappings; too complex and nobody will create tuning files. Must support intervals (not just note→opcode pairs) from the start.
**Delivers:** TOML-based tuning file format, parser, validator, default piano tuning, at least one interval-based tuning example.
**Features:** Configurable tuning files, default piano tuning, interval-based mapping.
**Pitfalls to address:** Tuning format expressiveness (#12).

### Phase 3: MIDI Input Pipeline
**Rationale:** MIDI is the simplest, cleanest input path — discrete events, no signal processing. Provides the first end-to-end "play music → see computation" experience. Validates the `MusicalEvent` abstraction and tuning engine integration.
**Delivers:** MIDI device connection, message parsing, event unification, full MIDI→Tuning→VM pipeline.
**Features:** MIDI input, note-on/off handling, velocity support.
**Pitfalls to address:** Note-On velocity 0 (#14), MIDI debouncing (#19), device enumeration (#9).

### Phase 4: CLI Visualization (TUI)
**Rationale:** At this point there's a working MIDI→VM pipeline. Adding visualization makes everything debuggable and demo-able. The visual feedback loop is what makes the performance compelling. Must be on a separate thread to avoid blocking the event pipeline.
**Delivers:** ratatui-based TUI showing stack state, last operation, output buffer, event log. Keyboard handling (quit, reset).
**Features:** CLI visualization, stack display, operation log, output display.
**Pitfalls to address:** Rendering blocking events (#15), fixed-framerate decoupling.

### Phase 5: Audio Capture & Pitch Detection
**Rationale:** The hardest component — signal processing, lock-free ring buffer architecture, onset detection, noise gating, musical gesture state machine. Everything downstream (VM, tuning, TUI) is already solid, so iteration on audio processing is low-risk.
**Delivers:** Microphone capture via cpal, lock-free audio pipeline, pitch detection (McLeod), onset detection, noise gate, gesture state machine, configurable buffer/window sizes.
**Features:** Microphone input, real-time pitch detection, onset detection, noise gating.
**Pitfalls to address:** Audio callback allocation (#1), mutex locks (#2), octave errors (#3), onset detection (#4), buffer size (#5), noise floor (#10), sample rate assumptions (#13), latency accumulation (#11).

### Phase 6: Integration, Polish & Distribution
**Rationale:** All components are individually tested. This phase wires the microphone path, adds default tunings, creates example programs, writes the README, and prepares for `cargo install` distribution. The Hello World demo is the capstone deliverable.
**Delivers:** Full mic+MIDI integration, voice/guitar tunings, Hello World + FizzBuzz examples, README, `cargo install` readiness, cross-platform testing.
**Features:** Dual input mode, instrument-specific tunings, example programs, README, distribution.
**Pitfalls to address:** Cross-platform device issues (#9), compile time/binary size (#20), latency budget validation (#11).

### Phase Ordering Rationale

The build order follows a strict dependency chain where each phase delivers a testable increment:
1. **VM first** — no external deps, defines core types, fully unit-testable
2. **Tuning second** — depends only on `Instruction` from VM, testable with synthetic events
3. **MIDI third** — simplest audio input, first end-to-end playable prototype
4. **TUI fourth** — makes everything visible, enables debugging of subsequent audio work
5. **Audio/pitch fifth** — hardest component, benefits from stable downstream pipeline
6. **Integration last** — wiring, polish, and content when all components are proven

This order minimizes blocked work and ensures a working demo (MIDI-only) exists halfway through development.

### Research Flags

| Phase | Research Needed? | Notes |
|-------|-----------------|-------|
| Phase 1: Language Spec & VM | **YES — `/gsd-research-phase`** | Jump/loop semantics in a live-input context is a novel design problem. Needs careful spec work. Standard stack VM patterns are well-documented, but the live-input twist requires original design decisions. |
| Phase 2: Tuning Files | Minimal | TOML parsing is well-documented. The interval-based mapping model is described in research. Focus on format design, not technology. |
| Phase 3: MIDI Input | No | `midir` is well-documented with examples. MIDI protocol is a 40-year-old standard. |
| Phase 4: CLI/TUI | No | `ratatui` has extensive examples and documentation. Standard patterns apply. |
| Phase 5: Audio & Pitch | **YES — `/gsd-research-phase`** | Pitch detection algorithm tuning, onset detection strategies, and the gesture state machine need empirical iteration. The `pitch-detection` crate is stable but unmaintained — may need evaluation against `pitch` crate (BCF algorithm) or hand-rolled YIN. |
| Phase 6: Integration | No | Standard wiring and polish. Cross-platform testing is the main effort. |

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | **HIGH** | All recommended crates are mature, actively maintained, high download counts. `pitch-detection` is the weakest link (unmaintained) but the algorithm is mathematically stable. Pure-Rust stack avoids C FFI build issues. |
| Features | **HIGH** | Feature landscape is well-defined by surveying esolang wiki, live-coding tools, and PROJECT.md. Clear P0/P1/P2 prioritization. Anti-features are firmly identified. |
| Architecture | **HIGH** | Three-thread model follows established real-time audio patterns (Ross Bencina, ADC talks). Lock-free communication, instruction budgeting, and gesture state machines are proven approaches. |
| Pitfalls | **HIGH** | 20 pitfalls identified with concrete prevention strategies. Critical pitfalls (audio callback allocation, mutex locks, octave errors, onset detection) are well-documented in DSP literature. |

### Gaps to Address During Planning

1. **Jump/loop semantics** — The language spec must resolve how jumps work in a live-input context before Phase 1 implementation. The buffer model, label model, and structured-loop-only model each have tradeoffs. This is the single most important design decision.
2. **`pitch-detection` crate viability** — Unmaintained for 3+ years. May need to fork or switch to `pitch` crate (BCF algorithm) or hand-rolled YIN if bugs are found. Evaluate during Phase 5 planning.
3. **Interval detection window timing** — The 200-500ms window that distinguishes "two notes forming an interval" from "two independent pushes" is a UX-critical parameter. Needs empirical tuning with real performers.
4. **Sustained note semantics** — What does holding a note do? Repeated pushes? Nothing? This is a language design question that affects both the spec and the onset/offset detection system.
5. **Cross-platform audio testing** — cpal and midir have platform-specific quirks. Linux (ALSA vs PipeWire), Windows (WASAPI driver variability), and macOS (CoreAudio aggregate devices) all need testing. Budget time for this in Phase 5-6.

## Sources

### From STACK.md
- cpal v0.17.3 — 11.6M downloads, cross-platform audio I/O
- midir v0.10.3 — 416K downloads, cross-platform MIDI
- midly v0.5.3 — zero-copy MIDI parsing
- pitch-detection v0.3.0 — 39K downloads, McLeod/YIN algorithms
- rustfft v6.4.1 — 15.7M downloads, SIMD-accelerated FFT
- ringbuf v0.4.8 — 10.9M downloads, lock-free SPSC
- ratatui v0.30.0 — 21.7M downloads, terminal UI
- crossterm v0.29.0 — 113M downloads, terminal backend

### From ARCHITECTURE.md
- Real-time audio architecture patterns (Ross Bencina, Audio Developer Conference)
- rtrb v0.3.3 — realtime-safe SPSC ring buffer
- crossbeam-channel v0.5.15 — MPMC channels

### From PITFALLS.md
- de Cheveigné & Kawahara (2002) — YIN fundamental frequency estimator
- Mauch & Dixon (2014) — pYIN probabilistic pitch detection
- Wessel & Wright (2002) — latency perception thresholds (10-30ms for musical immediacy)
- MIDI 1.0 spec (1983) — Note On velocity 0 = Note Off

### From FEATURES.md
- Esolang wiki survey (Brainfuck, Befunge, Piet, Chef, Shakespeare)
- Live-coding environments (Sonic Pi, TidalCycles, SuperCollider, ChucK)
