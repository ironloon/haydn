# Stack Research

**Domain:** Real-time musical esoteric programming language (Rust)
**Researched:** 2026-03-28
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Rust** | stable (1.85+) | Language | Non-negotiable per PROJECT.md. Memory safety, zero-cost abstractions, and the `cpal`/`midir` ecosystem make Rust the right call for real-time audio |
| **cpal** | 0.17.3 | Audio I/O (microphone capture) | *The* Rust audio I/O library. 11.6M downloads, actively maintained (last release ~1 month ago). Cross-platform: WASAPI (Windows), CoreAudio (macOS), ALSA (Linux). Pure Rust, no C FFI. [HIGH confidence] |
| **midir** | 0.10.3 | MIDI input capture | Cross-platform real-time MIDI I/O inspired by RtMidi. 416K downloads, updated 5 months ago. Supports ALSA, WinMM, CoreMIDI, JACK. Callback-based API matches real-time requirements. [HIGH confidence] |
| **midly** | 0.5.3 | MIDI message parsing | Zero-copy, fast MIDI decoder. Provides `LiveEvent::parse()` for real-time MIDI event parsing (NoteOn, NoteOff, CC, etc.). Use alongside `midir` — midir captures raw bytes, midly parses them into structured events. [HIGH confidence] |
| **pitch-detection** | 0.3.0 | Pitch detection (microphone) | Implements McLeod Normalized Square Difference Function (MPM) — best general-purpose monophonic pitch detector for real-time use. 39K downloads, most used pitch crate. Last updated 3+ years ago but the algorithms are mathematically stable; this is "done" software, not abandoned. Provides configurable power/clarity thresholds for noise rejection. [MEDIUM confidence — stable but unmaintained; may need to fork if bugs found] |
| **rustfft** | 6.4.1 | FFT computation | SIMD-accelerated (AVX/SSE/NEON/WASM), pure Rust FFT. 15.7M downloads. Needed if implementing custom pitch detection or doing spectral analysis for onset detection. [HIGH confidence] |
| **ratatui** | 0.30.0 | CLI visualization (stack state, output) | The dominant Rust TUI framework (successor to tui-rs). 21.7M downloads, very actively maintained (3 months ago). Rich widget set: Table (stack display), Paragraph (output), Gauge (volume), Block/Layout. [HIGH confidence] |
| **crossterm** | 0.29.0 | Terminal backend for ratatui | Cross-platform terminal manipulation. 113M downloads. Required backend for ratatui on all platforms. Handles raw mode, alternate screen, event polling. [HIGH confidence] |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **ringbuf** | 0.4.8 | Lock-free SPSC ring buffer | **Critical for real-time audio.** Pass audio samples from cpal's audio callback thread to the processing thread without blocking. Audio callbacks MUST NOT allocate, lock, or block. ringbuf provides the lock-free SPSC channel needed. 10.9M downloads. [HIGH confidence] |
| **serde** | 1.0.228 | Serialization framework | Deserialize tuning files (musical gesture → opcode mappings). Used with `toml` for config. [HIGH confidence] |
| **toml** | 1.1.0 | TOML config parsing | Tuning file format. TOML is human-readable, expressive enough for music mappings, and Cargo-ecosystem idiomatic. Updated 5 days ago. [HIGH confidence] |
| **clap** | 4.6.0 | CLI argument parsing | Parse CLI args: input source selection (MIDI/mic), tuning file path, device selection, verbosity. Derive-based API. Updated 15 days ago. [HIGH confidence] |
| **thiserror** | 2.0.18 | Library error types | Structured error enums for VM errors, parse errors, audio errors, MIDI errors. Use in library/core modules. [HIGH confidence] |
| **anyhow** | 1.0.102 | Application error handling | Use in `main()` and CLI layer for ergonomic error propagation with context. Pair with `thiserror` in library code. [HIGH confidence] |
| **tracing** | 0.1.44 | Structured logging/diagnostics | Structured, span-based logging. Critical for debugging real-time audio issues (latency spikes, dropped frames, pitch detection misses). Use `tracing-subscriber` for output formatting. [HIGH confidence] |
| **tracing-subscriber** | ~0.3 | Log output formatting | FmtSubscriber for stderr logging alongside ratatui's stdout TUI. Env-filter support for runtime log level control. [HIGH confidence] |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **criterion** 0.8.2 | Benchmarking | Benchmark pitch detection latency, VM instruction throughput, and audio buffer processing. Statistical analysis detects regressions. [HIGH confidence] |
| **cargo-flamegraph** | Profiling | Generate flamegraphs to find latency bottlenecks in the audio pipeline |
| **clippy** | Linting | Standard Rust linter; enforce with `#![deny(clippy::all)]` |
| **cargo-deny** | Dependency auditing | License compliance checking — critical since this is `cargo install` distributed |

## Architecture: Stack-Based VM Design

No crate needed — this is custom core logic. Recommended pattern:

```rust
// Opcode enum — each variant is a VM instruction
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Push(i64),
    Add, Sub, Mul, Div, Mod,
    Dup, Swap, Drop, Rotate,
    Eq, Gt, Lt,
    Jump(usize), JumpIf(usize),
    PrintNum, PrintChar,
    Read, Store(usize), Load(usize),
    Noop, Halt,
}

// VM state
pub struct Vm {
    stack: Vec<i64>,
    memory: Vec<i64>,
    output: Vec<u8>,
    pc: usize,  // only needed if using stored programs
}
```

For real-time musical input, the VM processes instructions one at a time as musical events arrive (not from a stored program). Each musical gesture maps through the tuning file to an `Opcode`, which executes immediately against the stack state.

**Key insight:** The VM doesn't need an instruction pointer or program counter in the traditional sense — it's event-driven. Each MIDI note or detected pitch triggers one or more opcodes that execute immediately.

## Thread Architecture

```
┌─────────────┐    ringbuf     ┌──────────────┐    channel    ┌────────────┐
│ Audio Thread │──────────────→│ Pitch Thread  │────────────→│  Main/UI   │
│ (cpal cb)   │  raw samples  │ (detection)   │  opcodes    │  Thread    │
└─────────────┘               └──────────────┘              │  (VM + TUI)│
                                                            └────────────┘
┌─────────────┐    channel    
│ MIDI Thread  │────────────→  (same Main/UI Thread)
│ (midir cb)  │  opcodes
└─────────────┘
```

- **Audio thread** (cpal callback): Fills ringbuf with PCM samples. MUST NOT allocate or block.
- **Pitch thread**: Reads samples from ringbuf, runs pitch detection, maps to opcodes via tuning config, sends opcodes via `std::sync::mpsc` channel.
- **MIDI thread** (midir callback): Parses MIDI events via midly, maps to opcodes, sends via channel.
- **Main thread**: Receives opcodes from both sources, executes VM, renders TUI via ratatui.

Use `std::sync::mpsc` for opcode channels (bounded, to apply backpressure). The ringbuf is specifically for the audio thread where lock-free guarantees are mandatory.

## Installation

```toml
# Cargo.toml

[package]
name = "haydn"
version = "0.1.0"
edition = "2021"
description = "An esoteric programming language performed by singing or playing music"
license = "MIT"
keywords = ["esolang", "music", "midi", "audio", "programming-language"]
categories = ["command-line-utilities"]

[dependencies]
# Audio I/O
cpal = "0.17"
midir = "0.10"
midly = "0.5"

# Pitch detection
pitch-detection = "0.3"
rustfft = "6.4"               # if custom pitch detection needed

# Lock-free audio communication
ringbuf = "0.4"

# TUI visualization
ratatui = "0.30"
crossterm = "0.29"

# Config / tuning files
serde = { version = "1.0", features = ["derive"] }
toml = "1.1"

# CLI
clap = { version = "4.6", features = ["derive"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Diagnostics
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
criterion = { version = "0.8", features = ["html_reports"] }

[[bench]]
name = "vm_throughput"
harness = false

[[bench]]
name = "pitch_latency"
harness = false
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `cpal` | `portaudio-rs` | Never for this project. portaudio-rs is C FFI bindings, harder to build, less maintained. cpal is pure Rust and better in every dimension. |
| `cpal` | `rodio` | If you need audio *playback* with high-level abstractions (decoding MP3s, mixing). Haydn is input-only; rodio adds unnecessary complexity. |
| `midir` | `jack` crate | If targeting pro-audio Linux setups only. JACK provides lower latency but limits portability. midir supports JACK as optional backend anyway. |
| `pitch-detection` (McLeod) | Hand-rolled YIN | If McLeod doesn't perform well enough on a specific instrument. YIN is ~200 lines to implement. Consider if pitch-detection's unmaintained status becomes a real problem. |
| `pitch-detection` | `aubio` (Rust bindings) | **Never.** GPL-3.0 licensed — incompatible with MIT distribution. Also requires C library build, last updated 5 years ago. |
| `pitch-detection` | `pitch` crate (BCF) | If BCF algorithm proves faster/more accurate for your instruments. Only 8.8K downloads and pre-1.0, but recently updated (MSRV 1.85) and uses an interesting Bitstream Autocorrelation approach. Worth evaluating in benchmarks. |
| `ratatui` | Raw `crossterm` only | If the TUI is truly minimal (just print lines). But stack visualization benefits enormously from ratatui's Table, Layout, and Block widgets. |
| `ratatui` | `iocraft` | If you prefer a React-like declarative TUI model. Newer, less battle-tested. Stick with ratatui for v1. |
| `toml` | `ron` (Rust Object Notation) | If tuning files need more complex data structures (nested enums). TOML is simpler and more familiar to users. |
| `ringbuf` | `rtrb` | `rtrb` (Real-Time Ring Buffer) is another SPSC option specifically designed for audio. Lower download count (~130K) but audio-focused. Either works; ringbuf is more battle-tested. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `aubio` / `aubio-rs` | GPL-3.0 license is incompatible with MIT. Requires C library compilation. Not updated in 5 years. | `pitch-detection` (pure Rust, MIT/Apache-2.0) |
| `tui` (original) | Unmaintained since 2022. Forked into `ratatui` which is actively developed. | `ratatui` |
| `portaudio-rs` | C FFI bindings, harder cross-platform builds, less maintained than cpal. | `cpal` |
| `rodio` | Playback-focused library. Adds audio decoding dependencies you don't need. Haydn only needs raw audio input. | `cpal` directly |
| `log` crate | Works but lacks structured spans. `tracing` is strictly better and is the modern Rust standard. | `tracing` |
| `tokio` / async runtime | Real-time audio MUST NOT use async. Audio callbacks are time-critical and cannot be multiplexed on an async executor. Use OS threads + lock-free channels. | `std::thread` + `ringbuf` + `mpsc` |
| `parking_lot` for audio thread | Any mutex (including parking_lot) can cause priority inversion on the audio thread. Audio callbacks must be 100% lock-free. | `ringbuf` (lock-free SPSC) |
| `serde_json` for tuning files | JSON lacks comments, is less human-readable for config. Musicians editing tuning files need a friendly format. | `toml` |

## Pitch Detection: Algorithm Notes

For Haydn's requirements (real-time, monophonic, musical input), the **McLeod Normalized Square Difference Function (MPM)** in `pitch-detection` is the recommended starting point:

- **Window size**: 1024–2048 samples at 44100 Hz (~23–46ms latency). 1024 is good for instruments above ~80 Hz (guitar, voice, keys). 2048 for bass instruments.
- **Hop size**: 512 samples (50% overlap) gives ~11ms update rate — sufficient for real-time responsiveness.
- **Power threshold**: Reject silent frames. Start at 5.0, tune per instrument.
- **Clarity threshold**: Reject ambiguous pitches. Start at 0.6–0.7. Lower for voice (more vibrato), higher for keyboard (cleaner signal).
- **MIDI input bypass**: MIDI input skips pitch detection entirely — notes are already discrete. This is the "easy mode" input path.

## Real-Time Latency Budget

| Stage | Target | Notes |
|-------|--------|-------|
| Audio buffer fill | 5–23ms | cpal buffer size (256–1024 samples at 44100 Hz) |
| Pitch detection | 1–5ms | McLeod on 1024 samples is fast |
| Opcode mapping | <0.1ms | Hash table lookup |
| VM execution | <0.1ms | Single stack operation |
| TUI render | ~16ms | 60fps target, non-blocking |
| **Total perceived** | **<50ms** | Under human perception threshold for "immediate" response |

For MIDI input, skip the audio+pitch stages — total latency is <1ms.

## Sources

- https://crates.io/crates/cpal — v0.17.3, 11.6M downloads, Apache-2.0 [verified on crates.io]
- https://crates.io/crates/midir — v0.10.3, 416K downloads, MIT [verified on crates.io]
- https://crates.io/crates/midly — v0.5.3, 205K downloads, Unlicense [verified on crates.io]
- https://crates.io/crates/pitch-detection — v0.3.0, 39K downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/rustfft — v6.4.1, 15.7M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/ringbuf — v0.4.8, 10.9M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/ratatui — v0.30.0, 21.7M downloads, MIT [verified on crates.io]
- https://crates.io/crates/crossterm — v0.29.0, 113M downloads, MIT [verified on crates.io]
- https://crates.io/crates/clap — v4.6.0, 735M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/serde — v1.0.228, 893M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/toml — v1.1.0, 547M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/thiserror — v2.0.18, 854M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/anyhow — v1.0.102, 601M downloads, MIT/Apache-2.0 [verified on crates.io]
- https://crates.io/crates/tracing — v0.1.44, 521M downloads, MIT [verified on crates.io]
- https://crates.io/crates/criterion — v0.8.2, 183M downloads, Apache-2.0/MIT [verified on crates.io]
- https://crates.io/crates/pitch — v0.2.0, 8.8K downloads, Apache-2.0/BSL-1.0/MIT [verified on crates.io, noted as backup option]
- https://crates.io/crates/aubio — v0.2.1, 13K downloads, GPL-3.0 [verified on crates.io, rejected: license]
