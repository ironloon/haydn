# Phase 6: Audio Capture & Pitch Detection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 06-audio-capture-pitch-detection
**Areas discussed:** Pitch detection approach, Real-time audio architecture, Onset & note segmentation, Crate placement & event model

---

## Pitch Detection Approach

| Option | Description | Selected |
|--------|-------------|----------|
| McLeod via `pitch-detection` crate | Battle-tested algorithm, Rust crate exists, handles harmonics well. Risk: crate flagged as unmaintained. | |
| YIN algorithm (hand-rolled) | Well-documented in literature (~200 lines of Rust), no external dependency risk, widely used in production pitch trackers. | |
| Agent's discretion | Let researcher evaluate both during planning. | |

**User's choice:** Hand-rolled, no external crate dependency.

**Follow-up: Algorithm baseline**

| Option | Description | Selected |
|--------|-------------|----------|
| YIN | Gold standard for monophonic pitch detection. de Cheveigné & Kawahara 2002. Parabolic interpolation gives sub-cent accuracy. | |
| McLeod NSDF | Similar autocorrelation family, normalized difference function, better harmonic handling. McLeod & Wyvill 2005. | |
| Agent's discretion | Let researcher compare during research phase. | |

**User's choice:** Implement both YIN and McLeod NSDF — benchmark against test audio and compare.

**Notes:** User preference for hand-rolled stems from eliminating unmaintained dependency risk and fully owning the DSP stack. Implementing both allows empirical comparison.

---

## Real-Time Audio Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Lock-free ring buffer (`ringbuf` crate) | SPSC ring buffer. cpal callback writes, analysis thread reads. Zero allocations, zero locks. Pro-audio standard pattern. | ✓ |
| Fixed-size `crossbeam` channel | Bounded channel with pre-allocated slots. Slightly higher overhead, MPMC (overkill for SPSC). | |
| Agent's discretion | Let researcher evaluate options. | |

**User's choice:** `ringbuf` SPSC — asked for recommendation, accepted the ringbuf rationale.

**Notes:** User explicitly stated preference for robustness: "I'd rather make this robust now than deal with a prob and have to fix/retrofit later." This aligns with STATE.md warning about lock-free architecture. Three-thread architecture (cpal → analysis → main) was presented and accepted as part of this decision.

---

## Onset & Note Segmentation

| Option | Description | Selected |
|--------|-------------|----------|
| Pitch-change-based onset | New note when pitch shifts >50 cents. Simple but misses repeated same-note attacks. | |
| Energy-based onset | RMS energy spike triggers new note. Catches repeated attacks but false-triggers on swells/vibrato. | |
| Combined energy + pitch | Energy spike as primary trigger, pitch confirms identity. Most robust. More tuning parameters. | ✓ |
| Agent's discretion | Let researcher design the state machine. | |

**User's choice:** Combined energy + pitch, with tuning-file-configurable parameters.

**Notes:** User's key insight: "this is something we want the tuning to be able to set to adjust as well." Different instruments need different onset sensitivity (vocal legato vs guitar pluck). Led to the `[audio]` section in tuning files — Phase 6 defines schema + piano defaults, Phase 7 authors voice/guitar values.

---

## Crate Placement & Event Model

### Crate placement

| Option | Description | Selected |
|--------|-------------|----------|
| New `haydn-audio` crate | Standalone library, parallel to haydn-vm and haydn-tuning. Testable in isolation. | ✓ |
| Inside `haydn` binary crate | Module in crates/haydn/src/audio/. Simpler structure but harder to test DSP in isolation. | |
| Agent's discretion | Let planner decide based on code volume. | |

**User's choice:** New `haydn-audio` crate — user noted the growing scope made a separate crate the right call.

### Event model

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror MidiMsg with AudioMsg | Separate `AudioMsg::NoteOn { note, confidence }`. Keep MIDI and audio paths independent. Converge at `process_note()`. | ✓ |
| Unified InputMsg enum | Single type replacing both MidiMsg and AudioMsg. Requires refactoring working MIDI code. | |
| Agent's discretion | Let planner decide type design. | |

**User's choice:** Separate AudioMsg, accepted recommendation to not touch MIDI code.

**Notes:** Recommendation rationale was: Phase 6 shouldn't refactor the working MIDI pipeline, confidence is semantically different from velocity, and unification is a mechanical refactor later if needed.

---

## Agent's Discretion

- Exact `PitchDetector` trait design and method signatures
- Analysis window size and hop size (latency vs accuracy within <60ms budget)
- Ring buffer size
- Sample format handling (f32 vs i16 normalization)
- Whether to use `rustfft` for spectral onset detection or stick with energy-based
- Note state machine internal states
- Audio device selection CLI flags
- Test strategy for pitch detection accuracy
- Whether confidence maps to velocity or is a separate field

## Deferred Ideas

- Unified `InputMsg` type (merge MidiMsg + AudioMsg) — mechanical refactor, not needed now
- Polyphonic pitch detection — v2 concern (ADV-01)
- Spectral flux onset detection — accuracy improvement for future phase
- cpal upgrade to 0.17+ — only if needed
