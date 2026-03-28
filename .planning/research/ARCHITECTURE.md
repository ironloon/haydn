# Architecture Research

**Domain:** Real-time musical esoteric language interpreter (Rust)
**Researched:** 2026-03-28
**Confidence:** HIGH

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         HAYDN RUNTIME                              │
│                                                                    │
│  ┌──────────────┐  ┌──────────────┐                                │
│  │  Microphone   │  │  MIDI Input  │       INPUT LAYER              │
│  │  (cpal)       │  │  (midir)     │                                │
│  └──────┬───────┘  └──────┬───────┘                                │
│         │ lock-free        │ channel                                │
│         │ ring buffer      │                                        │
│         │ (rtrb)           │                                        │
│  ┌──────▼───────┐  ┌──────▼───────┐                                │
│  │ Pitch/Onset  │  │ MIDI Message │       DETECTION LAYER           │
│  │ Detection    │  │ Parser       │                                │
│  └──────┬───────┘  └──────┬───────┘                                │
│         │                  │                                        │
│         └──────┬───────────┘                                        │
│                │                                                    │
│         ┌──────▼───────┐                                            │
│         │MusicalEvent  │              UNIFICATION LAYER             │
│         │(unified type)│                                            │
│         └──────┬───────┘                                            │
│                │                                                    │
│         ┌──────▼───────┐                                            │
│         │ Tuning File  │              MAPPING LAYER                 │
│         │ Lookup       │                                            │
│         └──────┬───────┘                                            │
│                │                                                    │
│         ┌──────▼───────┐                                            │
│         │ Stack-based  │              EXECUTION LAYER               │
│         │ VM           │                                            │
│         └──────┬───────┘                                            │
│                │ state snapshot                                      │
│         ┌──────▼───────┐                                            │
│         │ CLI/TUI      │              PRESENTATION LAYER            │
│         │ (ratatui)    │                                            │
│         └──────────────┘                                            │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **Audio Capture** | Open microphone, stream raw PCM samples to processing thread | `cpal` callback → `rtrb` ring buffer |
| **MIDI Capture** | Open MIDI port, receive discrete MIDI messages | `midir` callback → `crossbeam-channel` |
| **Pitch Detector** | Analyze audio buffer windows, extract fundamental frequency + onset | `pitch-detection` crate (McLeod or YIN algorithm) |
| **MIDI Parser** | Convert raw MIDI bytes to note-on/off, velocity, CC events | Custom (MIDI spec is simple: status byte + data bytes) |
| **Event Unifier** | Normalize both input types into a common `MusicalEvent` enum | Custom trait + enum |
| **Tuning Engine** | Map `MusicalEvent` → `Instruction` using loaded tuning file | `serde` + `toml` deserialization, HashMap lookup |
| **VM (Interpreter)** | Execute instructions on stack machine, manage memory/state | Custom stack-based VM (no dependencies) |
| **CLI Visualization** | Render stack state, current op, program output in terminal | `ratatui` + `crossterm` |
| **CLI App Shell** | Parse arguments, select input source, load tuning file, orchestrate | `clap` |

## Thread Architecture

This is the most critical architectural decision. Real-time audio imposes hard constraints.

### Three-Thread Model

```
┌─────────────────────────────────────────────────────────┐
│ AUDIO THREAD (OS-managed, high priority)                │
│ • cpal callback — receives raw f32 samples              │
│ • MUST NOT: allocate, lock, block, syscall              │
│ • Pushes samples into rtrb ring buffer                  │
│ • Runs at audio rate (e.g., 44100 Hz / buffer_size)     │
└────────────────────┬────────────────────────────────────┘
                     │ rtrb::Producer (lock-free, wait-free)
                     ▼
┌─────────────────────────────────────────────────────────┐
│ PROCESSING THREAD (dedicated std::thread)               │
│ • Pulls audio samples from rtrb::Consumer               │
│ • Runs pitch detection on accumulated windows           │
│ • Receives MIDI events via crossbeam channel            │
│ • Converts musical events → instructions via tuning     │
│ • Executes instructions on VM                           │
│ • Publishes VM state snapshots for UI                   │
│ • Runs in tight loop with ~1-5ms iteration              │
└────────────────────┬────────────────────────────────────┘
                     │ crossbeam-channel (state snapshots)
                     ▼
┌─────────────────────────────────────────────────────────┐
│ UI THREAD (main thread)                                 │
│ • ratatui render loop at ~30fps                         │
│ • Receives latest VM state snapshot                     │
│ • Handles keyboard input (quit, mode switches)          │
│ • Non-blocking: renders whatever state is available     │
└─────────────────────────────────────────────────────────┘
```

### Why Three Threads

| Concern | Solution |
|---------|----------|
| Audio callback must never block | Dedicated OS audio thread, write to lock-free ring buffer only |
| Pitch detection is CPU-intensive (~1ms per window) | Runs on processing thread, not in audio callback |
| VM execution could theoretically loop forever | Runs on processing thread with instruction budget per tick |
| TUI rendering is I/O-bound (terminal writes) | Separate thread so rendering never stalls audio processing |
| MIDI is already discrete events | Goes directly to processing thread via channel (no ring buffer needed) |

### Lock-Free Communication Details

**Audio → Processing:** `rtrb` SPSC ring buffer
- Producer in cpal callback, consumer in processing thread
- Wait-free: never blocks either side
- Buffer size: ~4096 samples (at 44100 Hz ≈ 93ms of audio — enough to absorb scheduling jitter)

**MIDI → Processing:** `crossbeam-channel` bounded channel
- midir callback pushes MIDI events, processing thread receives
- Bounded channel prevents unbounded memory growth
- Capacity: ~256 events (far more than any human can generate)

**Processing → UI:** `crossbeam-channel` bounded(1) or `std::sync::mpsc`
- Processing thread sends `VmStateSnapshot` structs
- UI thread grabs latest, drops intermediate frames
- Uses `try_recv()` loop to drain to latest — no back-pressure needed

## Recommended Project Structure

```
haydn/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point, argument parsing, orchestration
│   ├── lib.rs                  # Public API (for library consumers)
│   │
│   ├── vm/
│   │   ├── mod.rs              # VM public interface
│   │   ├── instruction.rs      # Instruction enum (push, add, sub, etc.)
│   │   ├── stack.rs            # Stack implementation with bounds
│   │   ├── memory.rs           # Addressable memory (store/load)
│   │   └── executor.rs         # Fetch-decode-execute loop
│   │
│   ├── tuning/
│   │   ├── mod.rs              # Tuning engine public interface
│   │   ├── file.rs             # TuningFile struct, loading, validation
│   │   └── mapping.rs          # MusicalEvent → Instruction mapping logic
│   │
│   ├── events/
│   │   ├── mod.rs              # MusicalEvent enum + traits
│   │   └── types.rs            # Note, Interval, Gesture types
│   │
│   ├── audio/
│   │   ├── mod.rs              # Audio subsystem setup
│   │   ├── capture.rs          # cpal device selection + stream setup
│   │   └── detector.rs         # Pitch detection + onset detection wrapper
│   │
│   ├── midi/
│   │   ├── mod.rs              # MIDI subsystem setup
│   │   └── capture.rs          # midir port selection + message parsing
│   │
│   └── ui/
│       ├── mod.rs              # UI subsystem setup
│       ├── app.rs              # App state for ratatui
│       ├── render.rs           # Layout and widget rendering
│       └── widgets.rs          # Custom widgets (stack display, event log)
│
├── tunings/
│   ├── default.toml            # Default keyboard tuning
│   ├── voice.toml              # Vocal tuning
│   └── guitar.toml             # Guitar tuning
│
├── examples/
│   ├── hello_world.toml        # Example program transcriptions
│   └── fizzbuzz.toml
│
└── tests/
    ├── vm_tests.rs             # VM instruction set tests
    ├── tuning_tests.rs         # Tuning file parse/map tests
    └── integration_tests.rs    # End-to-end with mock input
```

## Core Data Types

### MusicalEvent (Unification Layer)

```rust
/// A musical gesture detected from any input source.
pub enum MusicalEvent {
    NoteOn { pitch: MidiNote, velocity: f32, timestamp: Instant },
    NoteOff { pitch: MidiNote, timestamp: Instant },
    Interval { from: MidiNote, to: MidiNote, semitones: i8 },
    Rest { duration: Duration },
    // Future: dynamics, articulation, chord detection
}

/// MIDI note number (0-127), also used for quantized pitch detection output
pub struct MidiNote(pub u8);
```

### Instruction (VM Layer)

```rust
pub enum Instruction {
    Push(i64),          // Push value onto stack
    Add,                // Pop two, push sum
    Sub,                // Pop two, push difference
    Mul,                // Pop two, push product
    Div,                // Pop two, push quotient
    Mod,                // Pop two, push remainder
    Dup,                // Duplicate top of stack
    Swap,               // Swap top two elements
    Drop,               // Remove top element
    Rotate,             // Rotate top three elements
    Eq,                 // Pop two, push 1 if equal, 0 otherwise
    Gt,                 // Pop two, push 1 if greater, 0 otherwise
    Lt,                 // Pop two, push 1 if less, 0 otherwise
    Jump(usize),        // Jump to instruction address
    JumpIf(usize),      // Pop, jump if nonzero
    Loop(usize),        // Decrement top, jump if nonzero
    PrintNum,           // Pop and print as number
    PrintChar,          // Pop and print as ASCII char
    Read,               // Read input, push value
    Store(usize),       // Pop value, store at address
    Load(usize),        // Load from address, push value
}
```

### VmState (Processing → UI)

```rust
pub struct VmStateSnapshot {
    pub stack: Vec<i64>,            // Current stack contents
    pub last_instruction: Option<Instruction>,
    pub last_event: Option<MusicalEvent>,
    pub output_buffer: String,      // Accumulated text output
    pub instruction_count: u64,     // Total instructions executed
    pub memory_snapshot: Vec<(usize, i64)>,  // Non-zero memory cells
}
```

## Tuning File Architecture

Tuning files are the key differentiator — they decouple musical gesture from computation. The format must be human-readable, instrument-aware, and allow creative expression.

### Recommended Format: TOML

```toml
[meta]
name = "Piano Standard"
description = "Default tuning for keyboard/piano input"
instrument = "keyboard"

[notes]
# Single notes push their MIDI note number as a value
# This is the default behavior; override individual notes below
mode = "push_midi_value"

[intervals]
# Intervals (played as two successive notes) map to operations
# "ascending" = low then high, "descending" = high then low
1  = "dup"          # minor second
2  = "add"          # major second (whole step)
3  = "sub"          # minor third
4  = "mul"          # major third
5  = "div"          # perfect fourth
6  = "mod"          # tritone
7  = "swap"         # perfect fifth
8  = "drop"         # minor sixth
9  = "rotate"       # major sixth
10 = "eq"           # minor seventh
11 = "gt"           # major seventh
12 = "print_num"    # octave = output

[special]
# Special gestures
rest = "print_char"         # Silence for >500ms = print as char
trill = "loop"              # Rapid alternation = loop
staccato = "jump"           # Short detached note = jump
```

### Tuning Loading Pipeline

```
TOML file → serde deserialize → TuningFile struct → validate → TuningEngine
                                                                     │
MusicalEvent ──────────────────────────────────────────── lookup() ──►│──► Instruction
```

### Key Design Decision: Interval-Based Operations

Notes push values. Intervals between successive notes trigger operations. This means:
- Playing C4 pushes 60 onto the stack
- Playing E4 pushes 64 onto the stack (but if played within the interval window after C4, the 4-semitone interval triggers `mul` instead)
- The **interval detection window** (~200-500ms, configurable) determines whether two notes form an interval or are independent pushes

This is the fundamental mapping model. The tuning file can override any specific note or interval.

## Architectural Patterns

### Pattern 1: Audio Callback Discipline

The #1 rule of real-time audio: **never block in the audio callback.**

The cpal audio callback runs on a high-priority OS thread. Inside this callback:
- **DO:** Copy samples to ring buffer, increment counters
- **DON'T:** Allocate memory (`Vec::push`, `String`, `Box::new`), acquire locks (`Mutex`, `RwLock`), perform I/O (file, network, println), call anything that might block

```rust
// Inside cpal input callback:
move |data: &[f32], _: &cpal::InputCallbackInfo| {
    // GOOD: Write to pre-allocated ring buffer
    if let Ok(chunk) = producer.write_chunk(data.len()) {
        // copy samples — this is a memcpy, no allocation
        chunk.commit_all();
    }
    // If ring buffer is full, drop samples (acceptable trade-off)
}
```

### Pattern 2: Windowed Pitch Detection

Pitch detection requires a window of samples (typically 1024-2048 at 44.1kHz). The processing thread accumulates samples from the ring buffer into a fixed-size window buffer, runs detection, then slides the window.

```
Ring buffer → [accumulate in window buffer] → pitch detect → MusicalEvent
              ← slide window by hop_size →
```

- **Window size:** 2048 samples (~46ms at 44.1kHz) — good for voice/guitar fundamentals ≥ ~40Hz
- **Hop size:** 512 samples (~12ms) — determines how often we check for new pitches
- **Latency contribution:** ~46ms from windowing + ~12ms from hop = ~58ms perceived latency (acceptable for musical performance feel)

### Pattern 3: State Machine for Musical Gesture Detection

Not every detected pitch is a new note. The system needs a state machine to distinguish:

```
                    pitch detected
    SILENCE ──────────────────────► NOTE_ACTIVE
       ▲                                │
       │ silence timeout                │ pitch change > threshold
       │                                ▼
       │                          TRANSITION
       │                                │
       │ silence timeout                │ new pitch stabilizes
       │                                ▼
       └────────────────────────── NOTE_ACTIVE (new note)
```

States:
- **SILENCE:** No pitch detected (below power threshold). After timeout → emit Rest event.
- **NOTE_ACTIVE:** Stable pitch detected. Emit NoteOn, track duration.
- **TRANSITION:** Pitch changing, brief instability. Don't emit events yet.

### Pattern 4: Instruction Budget Per Tick

The VM must not run unbounded loops in a single processing tick. Use an instruction budget:

```rust
const INSTRUCTIONS_PER_TICK: usize = 1000;

fn tick(&mut self) -> VmStateSnapshot {
    for _ in 0..INSTRUCTIONS_PER_TICK {
        if self.halted || self.waiting_for_input {
            break;
        }
        self.step();
    }
    self.snapshot()
}
```

This prevents infinite loops from freezing the entire system. The performer can still create infinite loops — they just execute in bounded chunks, interleaved with audio processing.

## Data Flow: End-to-End

### Microphone Path (longest, ~60ms latency)

```
1. Sound wave hits microphone
2. OS audio driver captures samples
3. cpal callback fires with buffer of f32 samples        [AUDIO THREAD]
4. Samples pushed into rtrb ring buffer                   [AUDIO THREAD]
5. Processing thread pulls samples from ring buffer       [PROCESSING THREAD]
6. Samples accumulated into 2048-sample window            [PROCESSING THREAD]
7. McLeod pitch detector runs on window → frequency       [PROCESSING THREAD]
8. Frequency quantized to nearest MIDI note               [PROCESSING THREAD]
9. State machine emits MusicalEvent::NoteOn               [PROCESSING THREAD]
10. Tuning engine maps event → Instruction                [PROCESSING THREAD]
11. VM executes instruction, updates stack                [PROCESSING THREAD]
12. VmStateSnapshot sent via channel                      [PROCESSING THREAD]
13. UI thread receives snapshot, renders                  [UI/MAIN THREAD]
```

### MIDI Path (shortest, ~5ms latency)

```
1. Performer presses key on MIDI controller
2. MIDI driver delivers message
3. midir callback fires with bytes [status, note, vel]    [MIDI THREAD]
4. Message sent via crossbeam channel                     [MIDI THREAD]
5. Processing thread receives MIDI message                [PROCESSING THREAD]
6. Parsed to MusicalEvent::NoteOn                         [PROCESSING THREAD]
7. Tuning engine maps event → Instruction                 [PROCESSING THREAD]
8-10. (Same as microphone path steps 11-13)
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Mutex-Guarded Shared State
**What:** Using `Arc<Mutex<VmState>>` shared between audio/processing/UI threads.
**Why bad:** Mutex contention causes priority inversion on the audio thread. If the UI thread holds the lock while rendering, the audio callback blocks → audio glitches.
**Instead:** Message passing via channels and ring buffers. Each thread owns its data. State flows in one direction.

### Anti-Pattern 2: Dynamic Allocation in Audio Path
**What:** Creating `Vec`, `String`, or `Box` in the audio callback or in the hot processing loop.
**Why bad:** Allocator may lock internally, causing unbounded latency spikes.
**Instead:** Pre-allocate all buffers at startup. Use fixed-size arrays or ring buffers.

### Anti-Pattern 3: Blocking I/O in Processing Thread
**What:** Writing to stdout/stderr, reading files, or doing network I/O from the processing thread.
**Why bad:** I/O can block for milliseconds, causing audio input samples to accumulate and overflow.
**Instead:** All I/O happens on the UI thread. Processing thread only sends state snapshots.

### Anti-Pattern 4: Tight Coupling Between Input and VM
**What:** Having the MIDI parser directly call VM methods (e.g., `vm.execute(MidiEvent)`).
**Why bad:** Makes it impossible to swap input sources, test the VM independently, or add input processing stages.
**Instead:** All input sources produce `MusicalEvent`. The tuning engine converts to `Instruction`. The VM only knows about `Instruction`.

## Build Order

Based on dependency analysis, the optimal build order minimizes blocked work and maximizes testability at each step.

```
Phase 1: VM (no deps)
   │
Phase 2: Tuning Files (depends on: Instruction type from VM)
   │
Phase 3: MIDI Input (depends on: MusicalEvent type; easiest audio input)
   │      └── Integration: MIDI → Tuning → VM (first playable prototype)
   │
Phase 4: CLI Visualization (depends on: VmStateSnapshot from VM)
   │      └── Integration: MIDI → Tuning → VM → TUI (first visual prototype)
   │
Phase 5: Audio Capture + Pitch Detection (hardest component)
   │      └── Ring buffer architecture, windowed detection, state machine
   │
Phase 6: Integration + Polish
          └── Wire mic path, default tuning files, examples, docs
```

### Why This Order

| Phase | Rationale |
|-------|-----------|
| VM first | Zero external dependencies. Fully testable with hardcoded instructions. Defines the `Instruction` type everything else depends on. |
| Tuning second | Depends only on `Instruction` enum. Defines the mapping layer. Testable with synthetic `MusicalEvent`s. |
| MIDI third | Simplest audio input — MIDI messages are discrete, no signal processing needed. Provides first end-to-end "play music → see computation" path. |
| TUI fourth | By now there's a working VM with MIDI input. Visualization makes debugging and demos possible for all subsequent work. |
| Audio/pitch fifth | Hardest component (signal processing, ring buffer, state machine). Most likely to need iteration. By this point, everything downstream is solid. |
| Integration last | Wiring, polish, default tunings, example programs. All components tested individually first. |

## Scalability Considerations

| Concern | At Prototype | At v1.0 | Future |
|---------|-------------|---------|--------|
| Input latency | Acceptable (~60ms mic, ~5ms MIDI) | Tune window/hop sizes, measure end-to-end | Hardware-specific audio configs |
| VM throughput | 1000 instructions/tick | Profile, tune budget | JIT compilation if needed |
| Memory | Fixed-size stack (256), fixed memory (4096) | Configurable limits | Growable with warnings |
| Tuning complexity | Simple interval mapping | Chord detection, dynamics | ML-based gesture recognition |

## Sources

- **cpal** v0.17.3 — https://crates.io/crates/cpal (11.6M downloads, cross-platform audio I/O) — HIGH confidence
- **midir** v0.10.3 — https://crates.io/crates/midir (416K downloads, cross-platform MIDI) — HIGH confidence
- **pitch-detection** v0.3.0 — https://crates.io/crates/pitch-detection (39K downloads, McLeod/YIN/Autocorrelation) — MEDIUM confidence (smaller crate, but well-tested algorithms)
- **rtrb** v0.3.3 — https://crates.io/crates/rtrb (5.2M downloads, realtime-safe SPSC ring buffer) — HIGH confidence
- **crossbeam-channel** v0.5.15 — https://crates.io/crates/crossbeam-channel (385M downloads, MPMC channels) — HIGH confidence
- **ratatui** v0.30.0 — https://crates.io/crates/ratatui (21.7M downloads, terminal UI) — HIGH confidence
- Real-time audio architecture patterns — established audio programming discipline (see: Ross Bencina's "Real-time audio programming 101", Audio Developer Conference talks) — HIGH confidence
