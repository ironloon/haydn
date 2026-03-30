# Phase 6: Audio Capture & Pitch Detection - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Microphone input produces clean, discrete note events from continuous audio signal. This is the "messy" input path that complements MIDI's clean discrete events. The phase delivers a `haydn-audio` crate containing: real-time audio capture via cpal, hand-rolled pitch detection (YIN and McLeod NSDF), onset detection, noise gating, and a note state machine — all producing `AudioMsg` events that feed into the existing `process_note()` pipeline. Onset/noise parameters are tuning-file-configurable via a new `[audio]` section in the tuning schema.

This phase does NOT wire audio through the full Haydn pipeline (tuning → VM → TUI) — that integration happens in Phase 7. Phase 6 delivers the audio crate that produces verified note events from microphone input.

</domain>

<decisions>
## Implementation Decisions

### Pitch Detection Algorithm
- **D-01:** **Hand-rolled pitch detection, no external pitch detection crate.** No dependency on the `pitch-detection` crate (flagged as unmaintained in STATE.md). All DSP code is fully owned.
- **D-02:** **Implement both YIN and McLeod NSDF.** Two pitch detection algorithms behind a shared trait, benchmarked against test audio (synthetic waveforms and recorded samples). The researcher should reference the original papers (de Cheveigné & Kawahara 2002 for YIN, McLeod & Wyvill 2005 for NSDF). The planner decides how to structure the comparison.
- **D-03:** **Confidence threshold required.** Both algorithms must output a confidence score (0.0-1.0). Pitches below the confidence threshold are discarded as noise. Threshold is tuning-file-configurable.

### Real-Time Audio Architecture
- **D-04:** **`ringbuf` SPSC ring buffer** for the cpal audio callback → analysis thread path. Pre-allocated at startup (no runtime allocations in the audio callback). Zero locks, zero allocations in the real-time audio thread.
- **D-05:** **Three-thread architecture:**
  - Thread 1 (cpal callback): Writes raw f32 samples into the ring buffer. Real-time safe — no allocations, no locks, no blocking.
  - Thread 2 (analysis): Reads sample chunks from ring buffer, runs pitch detection + onset detection + noise gating, produces discrete `AudioMsg` events. Sends events over standard `mpsc` channel.
  - Thread 3 (main): Receives `AudioMsg` from channel, calls `process_note()` — same pattern as the MIDI pipeline.
- **D-06:** **cpal stays at current workspace version (0.15).** Upgrade to 0.17+ is agent's discretion if needed for input stream API improvements, but not required.

### Onset Detection & Note Segmentation
- **D-07:** **Combined energy + pitch onset detection.** Energy spike detection is the primary onset trigger (catches both pitch changes and repeated same-note attacks). Pitch detector confirms/updates note identity after onset is detected. Most robust approach for both voice and guitar.
- **D-08:** **Tuning-file-configurable parameters via `[audio]` section.** The tuning file gains an optional `[audio]` section with onset/noise parameters. Sensible defaults ship so it works without the section. Parameters include:
  - `noise_gate_db` — RMS threshold below which input is treated as silence
  - `onset_threshold_db` — energy spike magnitude to trigger a new note
  - `pitch_stability_cents` — pitch must hold within this range to confirm a note
  - `min_note_ms` — debounce / minimum time between note events
  - Agent's discretion for additional parameters the researcher identifies
- **D-09:** **Phase 6 defines the `[audio]` schema and ships piano defaults.** Voice and guitar parameter values are authored in Phase 7 (Audio Integration & Instrument Tunings).

### Crate Placement
- **D-10:** **New `haydn-audio` crate** in `crates/haydn-audio/`. Standalone library crate parallel to `haydn-vm` and `haydn-tuning`. Takes audio samples in, produces `AudioMsg` events out. Testable in isolation with synthetic waveforms.
- **D-11:** **Dependencies:** `cpal` (audio capture), `ringbuf` (lock-free SPSC buffer). No external pitch detection crates. The `haydn` binary crate adds `haydn-audio` as a dependency alongside `haydn-vm` and `haydn-tuning`.

### Event Model
- **D-12:** **`AudioMsg` enum parallel to `MidiMsg`.** `AudioMsg::NoteOn { note: u8, confidence: f32 }` and `AudioMsg::NoteOff`. Separate from `MidiMsg` — no refactoring of the working MIDI pipeline.
- **D-13:** **Convergence at `process_note()`.** Both `MidiMsg::NoteOn` and `AudioMsg::NoteOn` call the existing `process_note(note, velocity, engine, vm)` function. Audio confidence maps to the velocity parameter or is handled separately — agent's discretion on the exact mapping.
- **D-14:** **`haydn-audio` exports `start_audio_capture(device, audio_config) -> Receiver<AudioMsg>`.** The binary wires it up the same way it wires `midir`. A future unified `InputMsg` type is a possible refactor but not in scope for Phase 6.

### Agent's Discretion
- Exact `PitchDetector` trait design and method signatures
- Analysis window size and hop size (tradeoff between latency and accuracy — must stay within <60ms budget per AUD-06)
- Ring buffer size (e.g., 8192 or 16384 samples)
- Sample format handling (f32 vs i16 normalization)
- Whether to use `rustfft` for spectral onset detection or stick with energy-based
- Internal note state machine states (SILENCE → ONSET → ACTIVE → TRANSITION)
- Audio device selection CLI flags (`--mic-device`, `--audio-input`, etc.)
- Test strategy for pitch detection accuracy (synthetic sine waves, chirps, recorded samples)
- Whether confidence maps to velocity or is a separate field passed through the pipeline

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Specification (event interface contract)
- `docs/SPECIFICATION.md` §4 — Event interface: `Push(i64)` and `Op(Opcode)`. Audio must produce identical event types as MIDI. Source independence guarantee.
- `docs/SPECIFICATION.md` §6 — Silence and timing: silence is absence of events, no timing semantics, sustained notes are tuning-file-driven.

### VM Interface (the pipeline's consumer)
- `crates/haydn-vm/src/types.rs` — `Event`, `Opcode`, `StepResult` types — what the tuning engine produces from note events
- `crates/haydn-vm/src/lib.rs` — `HaydnVm::feed()`, `step()`, `process_event()` API

### Tuning Engine (the pipeline's mapper)
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine::map_note(u8) -> Option<Event>` — the function audio note events will call
- `crates/haydn-tuning/src/types.rs` — `RawTuningFile`, `Metadata` types — where `[audio]` section will be added
- `crates/haydn-tuning/tunings/piano.toml` — Default piano tuning (will gain `[audio]` defaults)
- `docs/TUNING.md` — Tuning file format documentation (will need `[audio]` section addition)

### Existing MIDI Pipeline (structural reference)
- `crates/haydn/src/lib.rs` — `MidiMsg` enum, `midi_callback()`, `process_note()` — the pattern `AudioMsg` mirrors
- `crates/haydn/src/main.rs` — Main event loop, device selection, CLI args — where audio input will be wired in Phase 7

### Existing Audio Code (reusable patterns)
- `crates/haydn-performer/src/main.rs` lines 176-295 — `list_audio_devices()`, `find_input_device()` — device enumeration and selection via cpal (reusable)
- `crates/haydn-performer/src/main.rs` lines 354-520 — Prototype audio capture stream and naive frequency estimator (reference only — NOT production-ready, uses Mutex)

### Prior Phase Decisions
- `.planning/phases/01-language-specification/01-CONTEXT.md` — D-04/D-05: event interface is `Push`/`Op`, tuning owns mapping; D-11: silence is no-op; D-13: sustained notes can trigger events (tuning-driven)
- `.planning/phases/02-vm-core/02-CONTEXT.md` — D-01/D-02: step-by-step VM with StepResult; D-10: event-driven streaming architecture
- `.planning/phases/03-tuning-system/03-CONTEXT.md` — D-01/D-02/D-03: rule-based engine with exact note, range, interval triggers; D-07: partial opcode coverage allowed
- `.planning/phases/04-midi-input-pipeline/04-CONTEXT.md` — D-01/D-03: mpsc channel architecture, main loop pattern; D-15/D-17: velocity captured but unused by tuning, sustained notes flagged as Phase 6 concern
- `.planning/phases/05-cli-visualization/05-CONTEXT.md` — D-03: `--quiet` flag for scripting fallback

### Algorithm References
- YIN: de Cheveigné & Kawahara (2002) "YIN, a fundamental frequency estimator for speech and music"
- McLeod NSDF: McLeod & Wyvill (2005) "A smarter way to find pitch"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `find_input_device(device_name: Option<&str>)` in `haydn-performer/src/main.rs` — cpal device selection by name substring. Can be adapted or copied to `haydn-audio`.
- `list_audio_devices()` in `haydn-performer/src/main.rs` — enumerates both input and output devices. Already handles the cpal host API.
- `cpal` 0.15 already in workspace — proven to build and work on this project.
- `MidiMsg` + mpsc channel + `process_note()` pattern in `haydn/src/lib.rs` — the exact architectural pattern `AudioMsg` mirrors.
- `note_name(midi: u8)` utility in `haydn/src/lib.rs` — MIDI note to string conversion, shared across input sources.

### Established Patterns
- Cargo workspace with specialized library crates (`haydn-vm`, `haydn-tuning`, `haydn-performer`) and a binary orchestrator (`haydn`).
- Channel-based IPC: callbacks send typed messages over `mpsc::Sender`, main thread receives and processes.
- Device selection: auto-detect single device, prompt on multiple, `--device` flag for override.

### Integration Points
- `haydn-audio` produces `AudioMsg` → `haydn` binary receives → calls `process_note()` → `haydn-tuning` maps → `haydn-vm` executes.
- Tuning file `[audio]` section parsed by `haydn-tuning`, read by `haydn-audio` at session start.
- Phase 7 wires everything together end-to-end. Phase 6 delivers the tested crate.

</code_context>

<specifics>
## Specific Ideas

- User wants robustness over expedience — "make this robust now rather than deal with a problem and have to fix/retrofit later." This drove the ringbuf and hand-rolled algorithm decisions.
- Audio onset parameters should be tuning-file-driven because different instruments have fundamentally different attack characteristics (vocal legato vs guitar pluck vs keyboard sustain).
- Implementing both YIN and McLeod allows empirical comparison during development — the better performer for voice/guitar becomes the default, but both remain available.

</specifics>

<deferred>
## Deferred Ideas

- **Unified `InputMsg` type** — merging `MidiMsg` and `AudioMsg` into a single enum. Not needed for Phase 6 (both converge at `process_note()`), but a natural refactor if the codebase evolves.
- **Polyphonic pitch detection** — Phase 6 is monophonic only. Chord detection from audio is a v2 concern (ADV-01).
- **Spectral flux onset detection** — more sophisticated than energy-based, would require `rustfft`. Energy + pitch is sufficient for v1; spectral flux is an accuracy improvement for a future phase.
- **cpal upgrade to 0.17+** — current 0.15 works. Upgrade only if input stream API improvements are needed.

</deferred>

---

*Phase: 06-audio-capture-pitch-detection*
*Context gathered: 2026-03-29*
