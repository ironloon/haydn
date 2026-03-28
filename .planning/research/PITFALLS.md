# Pitfalls Research

**Domain:** Real-time musical esoteric language (Rust)
**Researched:** 2026-03-28
**Confidence:** HIGH (real-time audio constraints are well-documented; pitch detection pitfalls are established in DSP literature; VM correctness issues are well-known from decades of language implementation)

---

## Critical Pitfalls

### Pitfall 1: Allocating Memory in the Audio Callback

**What goes wrong:** The audio callback (cpal's `build_input_stream` closure) runs on a real-time thread with strict timing deadlines (typically 1–10ms). Any heap allocation (`Vec::push`, `String::new`, `Box::new`, `format!()`) calls the system allocator, which may lock, page-fault, or take unbounded time. Result: audio glitches, dropped samples, and pitch detection receiving corrupted data.

**Why it happens:** Rust makes allocation invisible — `vec.push()` looks harmless. Developers write idiomatic Rust in the callback without realizing it allocates. Even `println!()` allocates and acquires a lock on stdout.

**How to avoid:**
- Pre-allocate all buffers before the stream starts. Pass fixed-size arrays or pre-allocated ring buffers into the callback.
- Use a lock-free ring buffer (e.g., `ringbuf` or `rtrb` crate) to pass audio data FROM the callback TO a processing thread. The callback only writes into the ring buffer.
- Never use `String`, `Vec` (unless capacity is pre-allocated and known to not exceed), `Box`, `HashMap`, `format!()`, or any I/O in the callback.
- Lint rule: audit every line in the audio callback for hidden allocations. `clone()` on a `String` allocates. `.collect::<Vec<_>>()` allocates.

**Warning signs:**
- Intermittent audio "pops" or "clicks" during performance
- Pitch detection randomly returning wrong values under load
- Problems that only appear on slower hardware or when CPU is busy
- Issues that vanish in debug mode (because timing changes)

**Phase to address:** First phase that touches audio input. This is THE most important constraint in the entire project — violating it makes everything downstream unreliable.

---

### Pitfall 2: Mutex Locks Between Audio Thread and Main Thread

**What goes wrong:** Using `Mutex<T>` to share state between the real-time audio callback and the processing/UI thread causes priority inversion. The audio thread (high priority) blocks waiting for the main thread (low priority) to release the lock. OS scheduler doesn't know the audio thread is urgent. Result: audio dropout, latency spike, missed notes.

**Why it happens:** `Arc<Mutex<T>>` is the idiomatic Rust way to share state between threads. It's the first thing developers reach for. It works fine in non-real-time contexts.

**How to avoid:**
- Use lock-free communication: `rtrb` (real-time ring buffer), `crossbeam-channel` (bounded, for non-audio side), or atomics for simple flags.
- Architecture: Audio callback → lock-free ring buffer → Processing thread → channel → UI thread. No locks touch the audio callback.
- For shared configuration (e.g., current tuning file settings), use atomics or `arc-swap` for read-mostly data.
- If you must share complex state, use triple-buffering or a single-producer/single-consumer queue.

**Warning signs:**
- Audio works fine in isolation but glitches when UI updates
- Latency varies unpredictably during performance
- Adding logging or visualization makes audio worse
- Works on macOS (CoreAudio is more forgiving) but fails on Linux/ALSA

**Phase to address:** Architecture phase. The data flow between threads must be designed lock-free from the start. Retrofitting lock-free communication into a mutex-based architecture is a near-rewrite.

---

### Pitfall 3: Pitch Detection Octave Errors

**What goes wrong:** The YIN algorithm (and most autocorrelation-based pitch detectors) frequently returns octave errors — detecting a note one octave too high or too low. A singer singing C4 gets detected as C3 or C5. Since Haydn maps notes to operations, an octave error means the WRONG OPCODE executes — a `push` becomes a `jump`, a `dup` becomes a `print`. The program goes haywire.

**Why it happens:** Fundamental frequency detection is inherently ambiguous with harmonics. The autocorrelation function has peaks at the fundamental AND at integer multiples (harmonics) and sub-multiples. YIN's threshold parameter trades off between octave errors and noise sensitivity — no single value works for all instruments and volumes.

**How to avoid:**
- Use pYIN (probabilistic YIN) instead of plain YIN — it uses a Hidden Markov Model to smooth transitions and reduce octave jumps. The `pitch-detection` Rust crate may offer this.
- Implement octave-jump filtering: if the detected pitch suddenly jumps exactly one octave, hold the previous pitch for a few frames and re-evaluate.
- Use the tuning file to define which octave range is valid for each instrument. A soprano voice won't produce C2 — clamp or reject out-of-range detections.
- Add hysteresis: once a pitch is established, require sustained evidence before changing by a large interval.
- For Haydn specifically: consider mapping by pitch CLASS (C, D, E...) rather than absolute pitch — this makes octave errors less catastrophic (C3 and C4 both map to "C" operations).

**Warning signs:**
- Sporadic wrong opcodes that can't be reproduced by playing the same note
- Programs that work with MIDI but fail with microphone on the "same" performance
- Octave-dependent instructions (like jump targets) producing wild behavior

**Phase to address:** Pitch detection implementation phase. Must be addressed before the microphone→VM pipeline is connected. Design the note mapping to be octave-error-resilient from the start.

---

### Pitfall 4: No Onset Detection (Relying Only on Pitch)

**What goes wrong:** Continuous pitch detection returns a pitch value for EVERY audio frame, even during sustained notes, silence, and transitions between notes. Without onset detection, the system either: (a) fires the same opcode 100+ times per second for a held note, or (b) fires random opcodes during pitch transitions and noise.

**Why it happens:** Pitch detection and onset detection are different problems solved by different algorithms. Developers implement pitch detection first and assume "I'll figure out when a note starts later." But the entire event model depends on onset detection.

**How to avoid:**
- Implement onset detection FIRST, before pitch detection. An onset detector tells you WHEN a new note begins. Pitch detection tells you WHAT the note is.
- Use spectral flux or energy-based onset detection. A new note = a sharp increase in energy or spectral change.
- Architecture: Onset detector triggers → pitch detector runs on the onset frame → result is a discrete note event → VM executes one instruction.
- Add a note-off detector: silence or energy below threshold for N frames = note ended.
- For sustained notes: decide the semantics. Does holding a note push the same value repeatedly? Does it do nothing? This is a LANGUAGE DESIGN decision that must be made early.

**Warning signs:**
- The VM runs operations at audio-frame rate (hundreds per second) instead of note rate (1-5 per second)
- Held notes cause stack overflow from rapid repeated pushes
- Performance feels "out of control" — performer can't stop execution

**Phase to address:** Audio input processing phase, BEFORE connecting to the VM. The event abstraction layer (raw audio → discrete musical events) is a critical architectural boundary.

---

### Pitfall 5: Audio Buffer Size Misconfiguration

**What goes wrong:** Buffer size directly controls the latency/stability tradeoff. Too small (64 samples) = constant buffer underruns and audio dropouts on most hardware. Too large (4096+ samples) = 90ms+ latency that makes the performer feel disconnected from the execution — they play a note, wait, then see the result. The "live performance" feel is destroyed.

**Why it happens:** Different platforms have different optimal ranges. macOS CoreAudio handles small buffers well. Linux ALSA without JACK may need larger buffers. Windows WASAPI varies wildly by driver. Developers test on their own machine and pick a buffer size that works for them.

**How to avoid:**
- Default to 512 samples (~11ms at 44.1kHz) as a reasonable starting point.
- Make buffer size configurable via CLI flag (e.g., `--buffer-size 256`).
- Document the tradeoff for performers: "Lower = more responsive, higher = more stable."
- Test on all three platforms with various audio interfaces, not just a laptop's built-in mic.
- Consider adaptive buffer sizing or at least detecting underruns and suggesting a larger buffer.

**Warning signs:**
- "It works on my machine" — works on developer's Mac, fails on user's Windows laptop
- Users report audio "crackling" that the developer can't reproduce
- Latency feels fine during development but performers complain during actual use

**Phase to address:** Audio infrastructure phase. Buffer size should be a configurable parameter from the start, not hardcoded.

---

### Pitfall 6: Treating Microphone Input Like MIDI

**What goes wrong:** MIDI gives you clean, discrete, timestamped note events: "Note On: C4, velocity 100, time 12345." Microphone gives you a continuous, noisy, ambiguous signal that must be INTERPRETED into note events. Developers design the event pipeline for MIDI first (because it's easier), then try to shove microphone data through the same interface. The abstraction doesn't hold — microphone events have uncertainty, MIDI events don't.

**Why it happens:** MIDI is clean and motivating to work with. It's natural to start there. But the microphone pipeline needs a completely different front-end before it can produce compatible events.

**How to avoid:**
- Design a `MusicalEvent` abstraction from the start that accommodates both sources:
  ```
  MusicalEvent { pitch: Note, confidence: f32, velocity: f32, source: InputSource, timestamp: Instant }
  ```
- The `confidence` field is critical — MIDI events get confidence 1.0, microphone events get the pitch detector's confidence value. The VM can optionally ignore low-confidence events.
- Build the microphone pipeline and MIDI pipeline as separate input adapters that both produce `MusicalEvent`s. Don't try to make one look like the other internally.

**Warning signs:**
- The `MusicalEvent` struct has no confidence/certainty field
- Microphone-triggered programs behave erratically while MIDI works perfectly
- The word "unwrap" appears anywhere in the microphone→event path

**Phase to address:** Architecture phase. The event abstraction is a load-bearing interface that everything else depends on.

---

### Pitfall 7: VM Stack Overflow / Underflow Without Bounds

**What goes wrong:** A performer holds a note (repeated pushes), and the stack grows unboundedly until the process OOMs. Or an operation pops from an empty stack and panics/UBs. In a live performance, a crash or freeze is catastrophic — the performer is on stage.

**Why it happens:** In normal programming languages, the programmer controls when pushes/pops happen. In Haydn, musical input drives the VM — the performer might not realize they're pushing faster than popping. The input rate is governed by human performance, but onset detection errors or held notes can produce unexpected event bursts.

**How to avoid:**
- Hard stack size limit (e.g., 1024 elements). Push to a full stack is a no-op (or wraps, or drops the bottom — define the behavior in the spec).
- Underflow is a no-op, not a crash. Popping an empty stack returns 0 or does nothing. Document this in the language spec.
- NEVER `panic!()` or `unwrap()` in any code path reachable from audio/performance input. Every error is recoverable.
- Consider a `stack_depth` indicator in the CLI visualization so the performer can see when they're getting deep.

**Warning signs:**
- `.unwrap()` or `.expect()` calls in the VM execution path
- No `MAX_STACK_SIZE` constant in the codebase
- Tests don't cover "what happens when the stack is full/empty"

**Phase to address:** VM implementation phase. Define overflow/underflow semantics in the language spec BEFORE implementing the VM.

---

### Pitfall 8: Infinite Loops With No Escape Mechanism

**What goes wrong:** The Haydn instruction set includes `jump` and `conditional` — meaning loops are possible (required for Turing completeness). A performer accidentally (or intentionally) creates an infinite loop. The VM spins, the CLI freezes, the audio callback may stall, and there's no way to break out except killing the process. During a live performance, this is a showstopper.

**Why it happens:** In normal esolangs, the developer runs a program and can Ctrl+C. In Haydn, the program IS the performance — the performer needs to break out of a loop by playing a specific musical gesture, not by reaching for the keyboard.

**How to avoid:**
- Implement an iteration limit per loop (configurable, default 10000). When exceeded, break out and continue execution.
- OR: implement a "break" gesture — a specific musical pattern (e.g., a rest followed by a high note) that interrupts any running loop.
- VM execution runs on its own thread with a yield/check mechanism — it checks for new input events periodically, so the performer can always override.
- At minimum: a CLI keybinding (e.g., Escape) that resets the VM state without killing the process.

**Warning signs:**
- No timeout or iteration limit on loops
- The VM execution blocks the thread that reads new input
- No test for "what happens during an infinite loop"

**Phase to address:** VM implementation phase and language spec. Loop semantics and escape mechanisms are language design decisions.

---

### Pitfall 9: Cross-Platform Audio Device Enumeration Failures

**What goes wrong:** `cpal` enumerates audio devices differently on each platform. On Linux, PulseAudio/PipeWire/ALSA present different device lists. On Windows, WASAPI may not see some devices. On macOS, CoreAudio usually just works but aggregate devices cause issues. The application crashes or shows "no audio device found" for users who definitely have a microphone.

**Why it happens:** Audio device management is one of the most platform-specific areas in computing. `cpal` abstracts a lot but edge cases leak through: devices with unusual sample rates, exclusive-mode-only devices, hot-plugged USB interfaces, virtual audio devices.

**How to avoid:**
- Use `cpal`'s default input device as the, well, default — but let users specify a device by name via CLI flag.
- List available devices with a `--list-devices` command so users can troubleshoot.
- Handle device enumeration errors gracefully — if the default fails, show a helpful error with `--list-devices` suggestion.
- Test on all three platforms. Linux requires testing with both PulseAudio and PipeWire (PipeWire is now default on Fedora, Ubuntu 24+).
- For MIDI: `midir` has similar per-platform quirks. Virtual MIDI ports on macOS work differently than on Linux/Windows.

**Warning signs:**
- Only tested on one OS
- Device enumeration uses `.unwrap()` instead of error handling
- No `--list-devices` or `--device` CLI flags
- Bug reports from Linux users with PipeWire

**Phase to address:** Audio infrastructure phase. Device handling should be robust from the start.

---

### Pitfall 10: Noise Floor and Ambient Sound Triggering False Events

**What goes wrong:** In a live performance environment, there's ambient noise — audience, HVAC, other instruments, feedback. The microphone picks up everything. Without proper noise gating, every ambient sound is interpreted as a musical event and triggers VM operations. The program runs garbage operations continuously.

**Why it happens:** Developers test in quiet rooms with good microphones. Live performance environments are dramatically different. The pitch detector will happily return a pitch for any periodic-ish signal, including HVAC hum (often around B1/60Hz in the US) and room resonance.

**How to avoid:**
- Implement an energy/amplitude gate: below a threshold dB level, ignore all input. Make this threshold configurable (`--noise-gate -40dB`).
- Add a calibration step at startup: "Please be silent for 3 seconds" → measure ambient noise level → set gate automatically.
- The gate should have attack/release characteristics (not just on/off) to avoid cutting off quiet notes that start below threshold.
- Consider a "confidence threshold" — the pitch detector's confidence output is lower for noise than for clear tonal input.

**Warning signs:**
- Programs execute operations even when nobody is playing
- HVAC hum (60Hz/B1 or 50Hz/G#1) detected as a constant note
- System works in quiet office but fails at a performance venue

**Phase to address:** Audio input processing phase, alongside onset detection. Noise gating is prerequisite to reliable microphone input.

---

### Pitfall 11: Latency Accumulation Across the Pipeline

**What goes wrong:** Each stage adds latency: audio buffer (11ms) + pitch detection window (23ms for 1024 samples at 44.1kHz) + onset detection lookahead (10-30ms) + VM execution (usually <1ms) + CLI rendering (varies). Total can easily reach 50-100ms+. At 100ms, performers feel disconnect — they play a note, and the response is noticeably late. Musical flow breaks down.

**Why it happens:** Each component is designed and tested in isolation. Each one's latency seems acceptable. But they're in series, and latencies add up.

**How to avoid:**
- Budget latency upfront: target ≤30ms total input-to-visual-feedback. This is the threshold where performers perceive "immediate."
- Measure end-to-end latency, not just per-component. Use a test signal → timestamp at entry → timestamp at display → measure delta.
- Pitch detection: use smaller FFT windows with overlap, not one big window. YIN can work on 1024 samples for most of the voice range (down to ~80Hz fundamental).
- CLI rendering: don't re-render on every event. Batch updates at 30-60fps regardless of event rate.
- Consider the pipeline in terms of samples: at 44.1kHz, 30ms = 1323 samples. That's your total budget.

**Warning signs:**
- No end-to-end latency measurement exists in the codebase
- Feel is "sluggish" but each individual component seems fast
- Adding features progressively makes the feel worse

**Phase to address:** Should be a cross-cutting concern from the architecture phase. Set a latency budget and track it.

---

### Pitfall 12: Tuning File Format That Can't Express Real Musical Mappings

**What goes wrong:** The tuning file format is designed too simplistically (e.g., just "note → opcode" pairs). Real musical mappings need intervals (a fifth up = add), sequences (three ascending notes = push then dup), dynamics (loud = one thing, soft = another), and timing (fast trill = loop). A flat mapping can't express these, making Haydn too limited for expressive performance.

**Why it happens:** The simplest thing that works for a demo is a 1:1 note-to-opcode mapping. This gets the system "working" but locks in a model that can't express the actual design intent of configurable musical gestures.

**How to avoid:**
- Design the tuning file format with these mapping types in mind from the start:
  - Single note → opcode (basic)
  - Interval → opcode (ascending fifth = add)
  - Sequence/pattern → opcode/sequence (three ascending notes = specific action)
  - Velocity/dynamics → modifier (loud vs soft variants)
- Don't over-engineer early, but keep the format extensible. TOML or YAML with clear sections. Simple mappings work now, complex ones can be added without breaking existing files.
- Provide at least 2 example tuning files that use DIFFERENT mapping strategies (e.g., one pitch-based for keyboard, one interval-based for voice).

**Warning signs:**
- Tuning file only supports `note = "opcode"` pairs
- No way to map intervals, sequences, or dynamics
- Only one example tuning file exists
- The format requires a computer science background to understand

**Phase to address:** Language design / tuning file specification phase. Before implementing the parser.

---

## Moderate Pitfalls

### Pitfall 13: Sample Rate Assumptions

**What goes wrong:** Code assumes 44100Hz sample rate. User's microphone runs at 48000Hz (common on many USB interfaces). Pitch detection returns wrong frequencies (everything shifted sharp/flat), buffer calculations are wrong, and timing is off.

**How to avoid:** Query the actual device sample rate from `cpal` and use it everywhere. Never hardcode 44100. Pass sample rate as a parameter to pitch detection and onset detection.

**Warning signs:** Pitch detection is consistently a few percent sharp or flat. Works on one machine, wrong on another.

**Phase to address:** Audio infrastructure phase.

---

### Pitfall 14: MIDI Note-Off vs Note-On Velocity 0

**What goes wrong:** Some MIDI controllers send "Note Off" messages when a key is released. Others send "Note On with velocity 0" (which means the same thing per the MIDI spec). Code that only checks for Note Off events misses half the "note released" signals, causing stuck notes in the event pipeline.

**How to avoid:** Always treat Note On with velocity 0 as Note Off. This is a well-known MIDI gotcha. The `midir` or `midi-msg` crate may handle this, but verify.

**Warning signs:** Some MIDI keyboards work, others have "stuck notes" that keep triggering operations.

**Phase to address:** MIDI input phase.

---

### Pitfall 15: CLI Rendering Blocking the Event Pipeline

**What goes wrong:** CLI visualization (printing stack state, current operation, output) uses `println!()` or complex TUI rendering on the same thread as event processing. When the terminal is slow (SSH, Windows Terminal with complex output), rendering blocks event handling. Notes are missed or delayed.

**How to avoid:** Separate the rendering thread from the event/VM thread. VM thread sends state snapshots to the renderer via a channel. Renderer runs at a fixed framerate (30-60fps), not on every event. Use a TUI library that supports efficient partial updates (e.g., `ratatui`).

**Warning signs:** Adding more visualization detail makes the system less responsive. SSH sessions perform worse than local.

**Phase to address:** CLI visualization phase.

---

### Pitfall 16: Testing Turing Completeness Without Proof

**What goes wrong:** The project claims Turing completeness but can't prove it. Running FizzBuzz and Hello World demonstrates specific capabilities, not computational universality. Without proof (e.g., reduction from a known Turing-complete system, or implementing a BF interpreter in Haydn), the claim is unverifiable.

**How to avoid:**
- The instruction set (push, add, sub, mul, div, mod, dup, swap, drop, rotate, eq, gt, lt, jump, conditional, loop, print, read, store, load) IS sufficient — it has arithmetic, comparison, conditional branching, loops, and memory (store/load). This is at least as powerful as Brainfuck.
- Formally prove it: show a mapping from Brainfuck to Haydn instructions. Each BF operation maps to one or more Haydn operations. Since BF is Turing-complete, Haydn is too.
- Include the proof in the language specification.

**Warning signs:** "We're Turing-complete because we have loops and conditionals" — that's necessary but not sufficient without store/load (unbounded memory access).

**Phase to address:** Language specification phase.

---

### Pitfall 17: Jump Target Semantics in a Live-Input Language

**What goes wrong:** In traditional esolangs, `jump` goes to a specific instruction address in a fixed program. In Haydn, the "program" is generated live — there's no instruction 42 until the performer has played 42 events. What does `jump 42` do when only 30 events have been played? What does `jump -5` do? Does the program buffer all past instructions and replay?

**Why it happens:** Jump/loop semantics are designed assuming a static program. Haydn doesn't have a static program.

**How to avoid:** This is one of the most important LANGUAGE DESIGN decisions. Options:
- **Buffer model:** Keep a buffer of recent N instructions. Jumps can only go back within the buffer. Forward jumps wait (or are no-ops).
- **Loop model:** Instead of arbitrary jumps, provide "repeat last N operations K times." This is more musically intuitive and avoids the address problem.
- **Label model:** Specific musical gestures mark "labels." Jumps go to labels, not addresses. More ergonomic.
- **No random jump:** Replace `jump` with structured control flow only (loop/break/if). Sacrifices nothing for Turing completeness if loop + conditional is sufficient.

**Warning signs:** Jump implemented with absolute addressing into a live instruction stream. Nobody has written a program that actually uses jumps/loops successfully.

**Phase to address:** Language specification phase. Must be resolved BEFORE VM implementation.

---

## Minor Pitfalls

### Pitfall 18: Pitch Detection Window Size vs Lowest Frequency

**What goes wrong:** Pitch detection needs at least 2 full cycles of the fundamental frequency to detect it. For C2 (65Hz), that's ~30ms of audio (~1323 samples at 44.1kHz). If the detection window is 512 samples (~12ms), it can't detect anything below ~170Hz. Low-pitched instruments are deaf to the system.

**How to avoid:** Size the detection window based on the lowest expected pitch. For voice (down to ~80Hz), 1024 samples is minimum. For bass guitar (down to ~40Hz), need 2048+. Make this configurable or tied to the tuning file's expected range.

**Phase to address:** Pitch detection phase.

---

### Pitfall 19: Not Debouncing MIDI Input

**What goes wrong:** A keyboardist's fingers bounce on keys, producing rapid Note On/Off pairs for what was intended as a single keypress. Without debouncing, a single intended note fires 2-3 operations.

**How to avoid:** Ignore Note On events that occur within N ms of a Note Off on the same key (5-20ms debounce). This is standard MIDI handling.

**Phase to address:** MIDI input phase.

---

### Pitfall 20: cargo install Binary Size / Compile Time

**What goes wrong:** Pulling in heavy audio/DSP dependencies (cpal + its backends, FFTW bindings, etc.) causes: (a) long compile times for `cargo install` users (5-10+ minutes), (b) system library requirements that cargo can't install (ALSA dev libraries on Linux, etc.), and (c) large binary size.

**How to avoid:**
- Use pure-Rust dependencies where possible (avoid C library bindings).
- `cpal` is already pure Rust (uses system APIs via Rust bindings). Good.
- For FFT, use `rustfft` (pure Rust) rather than FFTW bindings.
- Document system prerequisites clearly: Linux users need `libasound2-dev` (ALSA), etc.
- Consider pre-built binaries via GitHub Releases in addition to `cargo install`.

**Phase to address:** Build/distribution phase.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `unwrap()` in VM execution path | Faster initial development | Crash during live performance on edge input | Never in audio/VM paths; fine in startup/config parsing |
| Hardcoded sample rate (44100) | One less parameter to pass around | Wrong pitch detection on 48kHz devices, subtle and hard to debug | Never — query from device from day 1 |
| Single-threaded audio+VM+render | Simpler architecture, faster prototype | Latency accumulation, render blocking audio, can't optimize independently | Only for a proof-of-concept that will be rewritten |
| `Mutex` shared state between threads | Idiomatic Rust, easy to write | Priority inversion, audio dropouts under load | Never for audio thread communication |
| Flat note→opcode tuning format | Quick to implement and parse | Can't express intervals, sequences, or dynamics; limits the language's expressiveness | OK for first prototype, but plan for extension |
| No noise gate on microphone input | Simpler pipeline, less configuration | Ambient noise triggers random operations in any non-silent environment | Never once microphone input is "real" |
| Synchronous CLI output (`println!`) | Easy, works immediately | Blocks processing thread, causes event loss under heavy output | Only during early debugging, remove before any demo |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| FFT/autocorrelation per audio frame | High CPU, thermal throttling on laptops | Perform pitch detection only after onset detection triggers, not continuously | Always, even on fast hardware — it wastes power |
| Re-rendering entire CLI on every event | Rendering takes longer than event interval at high note rates | Fixed-framerate rendering (30-60fps) decoupled from event rate | Fast passages with >10 notes/second |
| String formatting in hot paths | Allocation stalls, GC-like pauses | Pre-format display strings, use fixed buffers for frequent output | Sustained performance over 30+ seconds |
| Large pitch detection window with full overlap | CPU quadruples vs 50% overlap, latency increases | Use hop size of 50-75% of window (512 hop for 1024 window) | Multi-track or high-sample-rate scenarios |
| Unbounded instruction history for jump targets | Memory grows linearly with performance duration | Ring buffer with fixed capacity, or structured loops instead of arbitrary jumps | Long performances (5+ minutes at moderate tempo) |
| Blocking device enumeration at startup | 2-5 second startup delay on some systems | Enumerate asynchronously, show "detecting audio..." message | Windows with many audio devices, Bluetooth audio |

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Audio infrastructure (cpal setup) | Allocation in callback, cross-platform device issues | Lock-free ring buffer from audio callback; test on Win/Mac/Linux; configurable buffer size and device |
| Pitch detection | Octave errors, noise triggering, low-frequency cutoff | pYIN or filtered YIN; noise gate; configurable window size per pitch range |
| Onset detection | Missing onsets in quiet passages, false onsets from noise | Energy + spectral flux hybrid; calibration step; adjustable sensitivity |
| MIDI input | Note-on velocity 0, debouncing, jitter | Treat vel=0 as note-off; debounce filter; timestamp events at arrival |
| Language specification | Jump semantics in live context, sustained note semantics, Turing completeness proof | Define before implement; buffer/loop model for jumps; formal BF reduction proof |
| VM implementation | Stack overflow/underflow, infinite loops, panics | Hard stack limit; iteration limit on loops; no unwrap in execution path |
| Tuning files | Too-simple format, not extensible | Design for intervals/sequences/dynamics from start; TOML/YAML; multiple examples |
| CLI visualization | Rendering blocking events, refresh rate issues | Separate render thread; fixed framerate; efficient partial updates with ratatui |
| Integration / end-to-end | Latency accumulation across pipeline stages | Latency budget (≤30ms total); end-to-end measurement; optimize the bottleneck |
| Distribution | System library deps, long compile times, binary size | Pure-Rust deps (rustfft, cpal); document prerequisites; consider pre-built binaries |

---

## Sources

- Real-time audio programming constraints: well-established in audio engineering (Ross Bencina's "Real-time audio programming 101: time waits for nothing," ADC talks, Audio Developer Conference archives)
- YIN / pYIN pitch detection: de Cheveigné & Kawahara (2002) "YIN, a fundamental frequency estimator for speech and music"; Mauch & Dixon (2014) "pYIN: A fundamental frequency estimator using probabilistic locally-normal distributions"
- cpal Rust crate: documented behavior for audio callbacks and device enumeration
- MIDI specification: Note On with velocity 0 = Note Off is in the original MIDI 1.0 spec (1983)
- Turing completeness: standard reduction technique — mapping from known TC language (Brainfuck) to target language
- Latency perception thresholds: Wessel & Wright (2002), various studies suggest 10-30ms for perceived "immediacy" in musical contexts
