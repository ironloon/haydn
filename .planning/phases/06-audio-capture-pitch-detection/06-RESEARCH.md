# Phase 6: Audio Capture & Pitch Detection — Research

**Researched:** 2026-03-29
**Status:** Complete
**Domain:** Real-time audio DSP, pitch detection algorithms, lock-free audio architecture

## Executive Summary

Phase 6 adds microphone input to Haydn via a new `haydn-audio` crate. The core challenge is converting a continuous audio signal into discrete note events (like MIDI but from a mic). This requires three DSP components working in concert: pitch detection (what note is playing?), onset detection (when does a new note begin?), and noise gating (is anyone even playing?). All must run within a 60ms latency budget on a real-time audio thread architecture.

The key technical risks are: hand-rolling pitch detection correctly (YIN and McLeod NSDF are well-documented but subtle), getting the ring buffer threading right (zero allocations in the audio callback), and tuning onset detection parameters for voice and guitar (fundamentally different attack profiles).

## 1. Pitch Detection Algorithms

### 1.1 YIN (de Cheveigné & Kawahara, 2002)

YIN estimates fundamental frequency from the autocorrelation of a signal. It operates on a single window of audio samples and returns a frequency estimate with a confidence measure.

**Algorithm steps (6 total):**

1. **Difference function** — For each lag τ from 1 to W/2 (W = window size), compute:
   ```
   d(τ) = Σ (x[j] - x[j + τ])²   for j = 0..W/2
   ```
   This measures how similar the signal is to a shifted version of itself. At the true period, d(τ) approaches 0.

2. **Cumulative mean normalized difference (CMND)** — Normalize to remove the overall energy bias:
   ```
   d'(0) = 1
   d'(τ) = d(τ) / [(1/τ) · Σ d(j) for j=1..τ]
   ```
   This is the key YIN innovation — it eliminates octave errors that plague raw autocorrelation.

3. **Absolute threshold** — Find the first τ where d'(τ) < threshold (typically 0.1–0.2). Lower threshold = higher confidence required. The confidence score is `1.0 - d'(τ_best)`.

4. **Parabolic interpolation** — Refine the integer lag estimate by fitting a parabola through d'(τ-1), d'(τ), d'(τ+1) to get sub-sample accuracy.

5. **Frequency conversion** — `f0 = sample_rate / τ_interpolated`

**Implementation parameters:**
- Window size: 2048 samples at 44100 Hz = ~46ms. Sufficient for voice (80+ Hz) and guitar (82+ Hz, low E).
- Minimum frequency detectable: `sample_rate / (window_size / 2)` = 44100 / 1024 ≈ 43 Hz. Covers all voice and guitar fundamentals.
- Threshold: 0.15 is a good starting default. Configurable via `[audio]` section.

**Complexity:** O(W²/2) for the difference function (dominant cost). For W=2048, that's ~2M multiply-adds per window — easily fast enough.

**Rust implementation pattern:**
```rust
pub struct YinDetector {
    window_size: usize,
    threshold: f32,
    // Pre-allocated buffers
    diff: Vec<f32>,
    cmnd: Vec<f32>,
}

impl YinDetector {
    pub fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<PitchEstimate> {
        // 1. Compute difference function into self.diff
        // 2. Compute CMND into self.cmnd  
        // 3. Find first minimum below threshold
        // 4. Parabolic interpolation
        // 5. Convert to frequency, return with confidence
    }
}
```

### 1.2 McLeod NSDF (McLeod & Wyvill, 2005)

McLeod's Normalized Square Difference Function (NSDF) is a refinement over YIN with better octave accuracy and cleaner peak selection.

**Algorithm steps:**

1. **NSDF computation** — For each lag τ:
   ```
   nsdf(τ) = 2·r(τ) / [Σ x[j]² + Σ x[j+τ]²]
   ```
   where r(τ) is the standard autocorrelation. The NSDF is normalized to [-1, 1], with 1.0 at τ=0 (perfect match).

2. **Key maxima selection** — Find all positive peaks of nsdf(τ) where the function crosses zero from negative to positive ("key maxima"). The first key maximum above the clarity threshold is the pitch period.

3. **Parabolic interpolation** — Same as YIN step 4, refine the peak position.

4. **Frequency conversion** — Same as YIN step 5.

**Why McLeod over YIN:**
- NSDF's normalization handles amplitude variations better (voice dynamics)
- Key maxima selection is more robust against octave errors
- The clarity metric (peak height) is a more intuitive confidence score (0.0–1.0 directly)

**Why keep both:**
- YIN is simpler to implement and debug — good as a reference
- McLeod may perform better for voice, YIN may edge out for guitar (different harmonic profiles)
- Both behind a `PitchDetector` trait — runtime switchable, benchmark-comparable

**Complexity:** O(W²) naive, or O(W log W) using FFT-based autocorrelation. For real-time use at W=2048, naive is fine (~47ms computation time is <1ms on modern CPU). FFT optimization via `rustfft` is an option but not needed unless profiling shows a bottleneck.

### 1.3 PitchDetector Trait Design

```rust
pub struct PitchEstimate {
    pub frequency_hz: f32,
    pub midi_note: u8,         // Nearest MIDI note number
    pub cents_offset: f32,     // Deviation from nearest note (-50 to +50)
    pub confidence: f32,       // 0.0 to 1.0
}

pub trait PitchDetector: Send {
    fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<PitchEstimate>;
    fn name(&self) -> &str;
}
```

**Frequency to MIDI conversion:**
```
midi_note = 69 + 12 * log2(frequency / 440.0)
cents_offset = (midi_float - midi_note_rounded) * 100
```

### 1.4 Testing Strategy for Pitch Detection

**Synthetic test signals (no audio hardware needed):**
- Pure sine waves at known frequencies (A4=440Hz, C4=261.63Hz, E2=82.41Hz)
- Sine waves with harmonics (simulate voice/guitar timbre)
- Frequency sweeps (glissando) to test tracking
- Silence → note onset → sustain → release sequences
- Two consecutive different notes to test onset+pitch changes

**Accuracy targets:**
- Within ±1 semitone (100 cents) for confident detection: maps to correct MIDI note
- Confidence > 0.8 for pure sines, > 0.6 for harmonic signals
- No false detections on silence (< noise gate threshold)

## 2. Real-Time Audio Architecture

### 2.1 Three-Thread Model

```
┌──────────────┐    ringbuf (SPSC)    ┌──────────────┐    mpsc channel    ┌──────────────┐
│  Thread 1    │ ──── f32 samples ──→ │  Thread 2    │ ── AudioMsg ────→ │  Thread 3    │
│  cpal        │                      │  Analysis    │                    │  Main        │
│  callback    │                      │  thread      │                    │  thread      │
└──────────────┘                      └──────────────┘                    └──────────────┘
  REAL-TIME SAFE                        Normal priority                     Normal priority
  No alloc, no lock                     Pitch, onset, gate                  process_note()
  No blocking                           Runs on ring buffer data            Receives AudioMsg
```

### 2.2 ringbuf SPSC Integration (v0.4.8)

**Setup:**
```rust
use ringbuf::{traits::*, HeapRb};

let rb = HeapRb::<f32>::new(16384); // ~370ms at 44100Hz — generous buffer
let (prod, cons) = rb.split();
// prod → moved into cpal callback closure (Send)
// cons → moved into analysis thread
```

**In cpal callback (Thread 1):**
```rust
move |data: &[f32], _: &cpal::InputCallbackInfo| {
    // push_slice writes as many samples as fit, discards overflow
    // This is the ONLY operation in the callback — no allocation, no lock
    let _ = prod.push_slice(data);
}
```

**In analysis thread (Thread 2):**
```rust
let mut analysis_buffer = vec![0.0f32; window_size];
loop {
    // Read window_size samples (blocking-spin until available)
    let available = cons.occupied_len();
    if available >= hop_size {
        // Shift buffer left by hop_size, fill new samples from ring buffer
        analysis_buffer.copy_within(hop_size.., 0);
        cons.pop_slice(&mut analysis_buffer[window_size - hop_size..]);
        
        // Run DSP pipeline
        let pitch = detector.detect(&analysis_buffer, sample_rate);
        // ... onset, gate, state machine → maybe send AudioMsg
    } else {
        std::thread::sleep(Duration::from_millis(1));
    }
}
```

**Buffer sizing rationale:**
- Ring buffer: 16384 samples ≈ 371ms at 44100Hz. This handles OS scheduling jitter — if the analysis thread gets delayed by 200ms, no samples are lost.
- Analysis window: 2048 samples ≈ 46ms. Good balance between pitch accuracy (lower frequencies need longer windows) and latency.
- Hop size: 512 samples ≈ 11.6ms. Updates pitch estimate ~86 times/sec. Far under the 60ms latency budget.

### 2.3 cpal Input Stream API (v0.15)

**Key API pattern for audio capture:**
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_input_device().ok_or("No mic found")?;
let config = device.default_input_config()?;
let sample_rate = config.sample_rate().0;
let channels = config.channels() as usize;

// Build typed input stream (f32)
let stream = device.build_input_stream(
    &config.into(),
    move |data: &[f32], _: &InputCallbackInfo| {
        // Extract mono (first channel only) and push to ring buffer
        for chunk in data.chunks(channels) {
            let _ = prod.try_push(chunk[0]); // or push_slice for batch
        }
    },
    |err| eprintln!("Audio stream error: {}", err),
    None, // timeout
)?;
stream.play()?;
```

**Sample format handling:**
cpal may deliver i16 or f32 depending on platform/device. Use `default_input_config()` and handle both:
- f32: pass through directly
- i16: normalize to f32 by dividing by `i16::MAX as f32`

The `haydn-performer` crate already demonstrates this pattern in its loopback test (lines 354-520 of main.rs). Adapt that pattern but replace `Mutex<Vec<f32>>` with `ringbuf` producer.

### 2.4 Latency Budget

| Stage | Target | Notes |
|-------|--------|-------|
| Audio callback → ring buffer | <1ms | Lock-free push, no blocking |
| Ring buffer → analysis window fill | ~11.6ms | One hop size at 512 samples/44100Hz |
| Pitch detection (YIN or McLeod) | <2ms | 2048-sample window, pure arithmetic |
| Onset detection + noise gate | <0.5ms | RMS + threshold comparison |
| Note state machine decision | <0.1ms | Enum match |
| mpsc send → main thread receive | <1ms | Standard channel |
| **Total** | **~16ms** | **Well within 60ms budget** |

The dominant latency is the hop size — how often we run pitch detection. At 512 samples/44100Hz = 11.6ms, we get ~86 updates per second. This is the inherent tradeoff: smaller hop = lower latency but more CPU. 512 is the standard choice.

## 3. Onset Detection

### 3.1 Energy-Based Onset Detection

**Algorithm:** Compare the RMS energy of the current frame to a running average. A spike above a threshold triggers an onset.

```rust
fn compute_rms(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

// In analysis loop:
let current_rms = compute_rms(&analysis_buffer);
let current_db = 20.0 * current_rms.log10();

if current_db > noise_gate_db && (current_db - prev_db) > onset_threshold_db {
    // Onset detected! Pitch detector confirms note identity.
}
```

### 3.2 Pitch-Confirmed Onset

Energy detection catches onsets; pitch detection identifies the note. Combined:

1. **Energy spike detected** → mark potential onset
2. **Pitch detection runs** → if confident pitch found, confirm onset with MIDI note
3. **If pitch matches previous note** → check if enough time has passed (debounce via `min_note_ms`)
4. **If pitch is different** → immediate onset (note change)

This handles:
- **Repeated same note** (piano hammer, guitar pluck): energy onset triggers even if pitch is identical
- **Legato transition** (voice): pitch change detected even without energy spike
- **Vibrato/pitch wobble**: `pitch_stability_cents` prevents wobble from triggering false onsets

### 3.3 Note State Machine

```
    ┌─────────────────┐
    │     SILENCE      │
    │ (no signal)      │
    └────────┬─────────┘
             │ RMS > noise_gate_db
             ▼
    ┌─────────────────┐
    │     ONSET        │
    │ (energy spike)   │──── no pitch → back to SILENCE (after timeout)
    └────────┬─────────┘
             │ pitch confirmed (confidence > threshold)
             ▼
    ┌─────────────────┐
    │     ACTIVE       │──── emit NoteOn(note, confidence)
    │ (sustaining)     │
    └────────┬─────────┘
             │ RMS < noise_gate_db OR new onset
             ▼
    ┌─────────────────┐
    │   TRANSITION     │──── emit NoteOff
    │ (note ending)    │──── if new onset: go to ONSET
    └────────┬─────────┘     if silence: go to SILENCE
             │
             ▼ (back to SILENCE or ONSET)
```

**State machine outputs:**
- `NoteOn { note: u8, confidence: f32 }` — emitted when ONSET → ACTIVE transition happens with confirmed pitch
- `NoteOff` — emitted when ACTIVE → TRANSITION → SILENCE, or when a new different note starts (NoteOff old, NoteOn new)

## 4. AudioMsg Event Model

```rust
#[derive(Debug, Clone)]
pub enum AudioMsg {
    NoteOn { note: u8, confidence: f32 },
    NoteOff,
}
```

Parallel to `MidiMsg` in `haydn/src/lib.rs`. Both converge at `process_note()`:
- `MidiMsg::NoteOn { note, velocity }` → `process_note(note, velocity, engine, vm)`
- `AudioMsg::NoteOn { note, confidence }` → `process_note(note, confidence_to_velocity(confidence), engine, vm)`

**Confidence → velocity mapping:** Scale 0.0–1.0 confidence to 1–127 velocity range. Simple linear: `velocity = (confidence * 126.0) as u8 + 1`. This let existing tuning rules that might someday use velocity work consistently.

## 5. Tuning File `[audio]` Section

### 5.1 Schema Extension

Add an optional `[audio]` section to `RawTuningFile`:

```rust
// In haydn-tuning/src/types.rs
#[derive(Debug, Deserialize)]
pub struct RawTuningFile {
    pub metadata: Metadata,
    pub values: Option<ValuesSection>,
    pub operations: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<RawRule>,
    pub audio: Option<AudioSection>,  // NEW
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioSection {
    /// RMS threshold in dB below which input is silence (default: -40.0)
    #[serde(default = "default_noise_gate_db")]
    pub noise_gate_db: f32,
    
    /// Energy spike in dB above running average to trigger onset (default: 6.0)
    #[serde(default = "default_onset_threshold_db")]
    pub onset_threshold_db: f32,
    
    /// Cents range within which pitch is considered stable (default: 50.0)
    #[serde(default = "default_pitch_stability_cents")]
    pub pitch_stability_cents: f32,
    
    /// Minimum milliseconds between note events (default: 80)
    #[serde(default = "default_min_note_ms")]
    pub min_note_ms: u32,
    
    /// Pitch detection confidence threshold 0.0-1.0 (default: 0.7)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
    
    /// Pitch detection algorithm: "yin" or "mcleod" (default: "mcleod")
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}
```

### 5.2 Default Values (Piano Tuning)

```toml
[audio]
noise_gate_db = -40.0
onset_threshold_db = 6.0
pitch_stability_cents = 50.0
min_note_ms = 80
confidence_threshold = 0.7
algorithm = "mcleod"
```

These are sensible defaults for a generic scenario. Phase 7 will add voice-specific and guitar-specific tuning files with different values (e.g., voice: lower onset_threshold_db because legato, higher min_note_ms because syllables are longer).

### 5.3 Validation Rules

- `noise_gate_db`: must be between -80.0 and 0.0
- `onset_threshold_db`: must be between 1.0 and 20.0
- `pitch_stability_cents`: must be between 10.0 and 200.0
- `min_note_ms`: must be between 10 and 500
- `confidence_threshold`: must be between 0.1 and 1.0
- `algorithm`: must be "yin" or "mcleod"

## 6. Crate Structure

```
crates/haydn-audio/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API: start_audio_capture(), AudioMsg, AudioConfig
│   ├── capture.rs      # cpal stream setup, ring buffer, analysis thread spawn
│   ├── pitch/
│   │   ├── mod.rs      # PitchDetector trait, PitchEstimate, freq_to_midi()
│   │   ├── yin.rs      # YIN implementation
│   │   └── mcleod.rs   # McLeod NSDF implementation
│   ├── onset.rs        # Energy-based onset detection + debounce
│   ├── gate.rs         # Noise gate (RMS threshold)
│   ├── state_machine.rs # Note state machine (SILENCE → ONSET → ACTIVE → TRANSITION)
│   └── types.rs        # AudioMsg, AudioConfig, AudioSection re-export
└── tests/
    ├── pitch_yin.rs    # YIN accuracy on synthetic signals
    ├── pitch_mcleod.rs # McLeod accuracy on synthetic signals
    ├── onset.rs        # Onset detection on synthetic envelopes
    └── integration.rs  # Full pipeline: samples → AudioMsg
```

### 6.1 Public API

```rust
// haydn-audio/src/lib.rs

pub use types::{AudioMsg, AudioConfig};
pub use pitch::{PitchDetector, PitchEstimate, YinDetector, McLeodDetector};

/// Start audio capture from the given device, producing AudioMsg events.
/// Returns a Receiver that the main thread polls for note events.
/// The cpal stream handle is returned so the caller can keep it alive.
pub fn start_audio_capture(
    device: cpal::Device,
    config: AudioConfig,
) -> Result<(mpsc::Receiver<AudioMsg>, cpal::Stream), AudioError> {
    // 1. Get device config (sample rate, channels)
    // 2. Create ringbuf (HeapRb::<f32>::new(16384))
    // 3. Create mpsc channel for AudioMsg
    // 4. Build cpal input stream with producer
    // 5. Spawn analysis thread with consumer + mpsc sender
    // 6. Return (receiver, stream)
}
```

### 6.2 Dependencies

```toml
[dependencies]
cpal = "0.15"
ringbuf = "0.4"
thiserror = "2"

[dev-dependencies]
# No additional dev deps needed — synthetic test signals are pure math
```

## 7. Common Pitfalls and Mitigations

| Pitfall | Mitigation |
|---------|------------|
| Octave errors in pitch detection | CMND normalization (YIN) / NSDF normalization (McLeod) — both specifically designed to prevent this |
| Audio callback blocking/allocating | ringbuf SPSC is zero-alloc. Push returns immediately. Only operation in callback is `push_slice`. |
| Ring buffer overflow (analysis too slow) | 16384 sample buffer = 371ms headroom. If overflow occurs, newest samples overwrite oldest — graceful degradation, not crash |
| Noise triggering false notes | Noise gate (RMS threshold) is first check before pitch detection even runs |
| Vibrato causing note flickering | `pitch_stability_cents` parameter + `min_note_ms` debounce in state machine |
| Sample format mismatch | Handle both f32 and i16 from cpal, normalize to f32 before ring buffer |
| Mono extraction from stereo+ | Take first channel only: `data.chunks(channels).map(|c| c[0])` |
| Platform differences (WASAPI vs CoreAudio vs ALSA) | cpal abstracts this. The `haydn-performer` crate already proves cpal works in this workspace on Windows. |

## 8. What NOT to Do

| Avoid | Why | Do Instead |
|-------|-----|------------|
| `pitch-detection` crate | Unmaintained, per STATE.md blocker and D-01 | Hand-roll YIN + McLeod |
| `Mutex` in audio callback | Priority inversion, latency spikes | `ringbuf` SPSC (lock-free) |
| `aubio-rs` | GPL-3.0, incompatible with MIT license | Hand-roll DSP |
| `tokio` / async in audio | Cannot multiplex real-time audio on async executor | `std::thread` + ring buffer |
| FFT-based pitch detection (initially) | More complex, not needed for v1 accuracy targets | Time-domain YIN/McLeod first |
| Allocating `Vec`s per analysis frame | Heap allocation pressure in hot loop | Pre-allocate all buffers at startup |

## Validation Architecture

### Nyquist Validation Points

1. **Pitch accuracy** — YIN and McLeod both detect A4 (440Hz) from synthetic sine within ±1 semitone
2. **Silence rejection** — No AudioMsg emitted when input is below noise gate
3. **Onset detection** — NoteOn emitted within 60ms of synthetic note onset
4. **Ring buffer throughput** — No sample loss at 44100Hz stereo for 10 seconds of continuous capture
5. **State machine transitions** — SILENCE→ONSET→ACTIVE→TRANSITION→SILENCE cycle verified with synthetic envelope
6. **AudioMsg convergence** — AudioMsg::NoteOn calls process_note() and produces correct VM events
7. **Tuning `[audio]` section** — Existing piano.toml continues to parse; new `[audio]` section parses with defaults

---

*Phase: 06-audio-capture-pitch-detection*
*Research completed: 2026-03-29*
