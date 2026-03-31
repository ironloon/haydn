# Phase 7 Research: Audio Integration & Instrument Tunings

**Researched:** 2026-03-30
**Phase:** 07-audio-integration-instrument-tunings
**Status:** Complete

## Research Questions

1. What are optimal root note, value range, and operation assignments for a voice tuning?
2. What are optimal root note, value range, and operation assignments for a guitar tuning?
3. What `[audio]` parameter values suit voice vs guitar acoustics?
4. How should AudioMsg events integrate into the existing main loop and TUI?
5. What changes are needed to existing crates to support audio mode?

## 1. Voice Tuning Design

### Vocal Ranges (MIDI note numbers)

| Voice Type | Low | High | Notes |
|------------|-----|------|-------|
| Bass | E2 (40) | E4 (64) | Lowest male voice |
| Baritone | A2 (45) | A4 (69) | **Most common male voice** |
| Tenor | C3 (48) | C5 (72) | Highest modal male voice |
| Contralto | F3 (53) | F5 (77) | Lowest female voice |
| Mezzo-soprano | A3 (57) | A5 (81) | Middle female voice |
| Soprano | C4 (60) | C6 (84) | Highest female voice |

### Generic Unisex Range

Per D-07: "approximately A2-A4 range, ~2 octaves." The baritone range (A2-A4) is the most common male voice type and overlaps with the lower range of female voices (contralto/mezzo). A2 (45) to A4 (69) = 25 semitones.

**Problem:** Piano tuning uses 24 value notes + 19 operation notes = 43 notes across ~5 octaves. Voice has only 25 semitones. Need 19 for operations + some for values.

### Recommended Voice Layout

**Root note: 55 (G3)** — slightly below center of range, giving breathing room for operations above.

| Zone | MIDI Range | Notes | Count | Purpose |
|------|-----------|-------|-------|---------|
| Values | 45–54 (A2–F#3) | A2 B♭2 B2 C3 C#3 D3 E♭3 E3 F3 F#3 | 10 | Push -10 to -1 |
| Operations | 55–73 (G3–C#5) | 19 consecutive semitones | 19 | All 19 opcodes |
| **Total** | **45–73** | | **29** | **~2.4 octaves** |

The operation zone extends slightly above A4 (69) to C#5 (73). While A4 is the stated upper bound, reaching C#5 requires only a minor stretch — well within what most untrained adults can produce, especially briefly for triggering operations. The top few notes (B♭4 through C#5) should map to rarely-used operations.

### Voice Operation Assignments

Core operations (used most often) in low-to-mid range, rare operations at extremes:

| MIDI | Note | Opcode | Rationale |
|------|------|--------|-----------|
| 55 | G3 | add | Most basic arithmetic — comfortable pitch |
| 56 | A♭3 | sub | Paired with add |
| 57 | A3 | mul | Arithmetic group |
| 58 | B♭3 | div | Arithmetic group |
| 59 | B3 | mod | Arithmetic group |
| 60 | C4 | dup | Stack ops — middle C, memorable center |
| 61 | C#4 | swap | Stack ops |
| 62 | D4 | drop | Stack ops |
| 63 | E♭4 | rotate | Stack ops |
| 64 | E4 | eq | Comparison group |
| 65 | F4 | gt | Comparison group |
| 66 | F#4 | lt | Comparison group |
| 67 | G4 | print_char | I/O — high but reachable |
| 68 | A♭4 | print_num | I/O |
| 69 | A4 | read | I/O (top of stated range) |
| 70 | B♭4 | loop_start | Control flow — stretch zone |
| 71 | B4 | loop_end | Control flow |
| 72 | C5 | store | Memory — reached by effort |
| 73 | C#5 | load | Memory — highest note |

**Design note:** Frequently-used operations (arithmetic, stack ops, I/O) are in the comfortable G3-A4 core. Loops and memory — used less often in simple programs — are in the stretch zone above A4. This means casual vocal programs (push values, add, print) stay in the easy range.

### Voice `[audio]` Parameters

Per D-06: "relaxed parameters so a casual singer gets usable output immediately."

```toml
[audio]
noise_gate_db = -35.0          # Voice is moderate intensity, slightly less sensitive
onset_threshold_db = 4.0       # Vocal onsets are softer than instruments
pitch_stability_cents = 80.0   # Voice has natural vibrato/wobble (~50 cents)
min_note_ms = 120              # Singing holds notes longer; avoid ghost triggers
confidence_threshold = 0.5     # Be forgiving — voice is noisier than instruments
algorithm = "mcleod"           # McLeod NSDF works well for monophonic voice
```

**Precision dial metaphor:** These defaults are intentionally loose. Tightening `pitch_stability_cents` toward 30 demands operatic pitch control. Raising `confidence_threshold` toward 0.8 rejects breathy/noisy vocal input. The performer tweaks these to dial in their desired precision-vs-chaos balance.

## 2. Guitar Tuning Design

### Guitar Range

Standard tuning open strings: E2(40), A2(45), D3(50), G3(55), B3(59), E4(64).
With 20+ frets: E2(40) up to roughly C6(84), but common playing range is E2(40) to E5(76).
Most acoustic playing happens E2(40) to A4(69) — first 12 frets.

### Recommended Guitar Layout

**Root note: 52 (E3)** — one octave above lowest open string, natural guitar center.

| Zone | MIDI Range | Notes | Count | Purpose |
|------|-----------|-------|-------|---------|
| Values | 40–51 (E2–D#3) | 12 notes, lowest octave | 12 | Push -12 to -1 |
| Operations | 52–70 (E3–B♭4) | 19 notes, fretboard core | 19 | All 19 opcodes |
| **Total** | **40–70** | | **31** | **~2.6 octaves** |

The guitar's wider comfortable range (vs voice) gives a better budget. All 19 operations fit within the common playing range (below fret 12 on most strings).

### Guitar Operation Assignments

| MIDI | Note | Opcode | Rationale |
|------|------|--------|-----------|
| 52 | E3 | add | Most basic — open 4th string area |
| 53 | F3 | sub | Arithmetic |
| 54 | F#3 | mul | Arithmetic |
| 55 | G3 | div | Arithmetic — open 3rd string |
| 56 | A♭3 | mod | Arithmetic |
| 57 | A3 | dup | Stack ops |
| 58 | B♭3 | swap | Stack ops |
| 59 | B3 | drop | Stack ops — open 2nd string |
| 60 | C4 | rotate | Stack ops — middle C |
| 61 | C#4 | eq | Comparisons |
| 62 | D4 | gt | Comparisons |
| 63 | E♭4 | lt | Comparisons |
| 64 | E4 | print_char | I/O — open 1st string |
| 65 | F4 | print_num | I/O |
| 66 | F#4 | read | I/O |
| 67 | G4 | loop_start | Control flow |
| 68 | A♭4 | loop_end | Control flow |
| 69 | A4 | store | Memory |
| 70 | B♭4 | load | Memory |

**Design note:** Open strings (E2=40, A2=45, D3=50) fall in the value zone — strumming open strings pushes values. Operations start at E3 (fret 2 on D string, or fret 7 on A string), reachable on multiple strings. Print operations align with the open 1st string area (E4=64).

### Guitar `[audio]` Parameters

Guitar has sharper onsets, more stable pitch, and wider dynamic range than voice.

```toml
[audio]
noise_gate_db = -45.0          # Guitar can be quiet (fingerpicking); be sensitive
onset_threshold_db = 8.0       # Guitar has sharp, clear attacks — higher threshold reduces string noise
pitch_stability_cents = 40.0   # Guitar pitch is more stable than voice (no vibrato by default)
min_note_ms = 60               # Fast note changes possible (hammer-ons, pull-offs)
confidence_threshold = 0.75    # Guitar signal is cleaner, can demand higher confidence
algorithm = "mcleod"           # Works well for guitar fundamentals
```

## 3. Integration Architecture

### Main Loop Dual-Input Design

Current architecture (MIDI only):
```
MidiInput → midi_callback → mpsc::Sender<MidiMsg> → rx.recv_timeout() → process_note()
```

Target architecture (MIDI or Audio):
```
[--input midi]  MidiInput → midi_callback → mpsc::Sender<MidiMsg> → process_note()
[--input mic]   cpal → ringbuf → analysis → mpsc::Sender<AudioMsg> → process_note()
```

Both paths converge at `process_note(note, velocity, engine, vm)`. Audio confidence maps to velocity: `(confidence * 127.0) as u8`.

### CLI Changes

New flags added to `Cli` struct:
- `--input midi|mic` — default `midi`, selects input source
- `--audio-device "name"` — substring match for microphone selection (parallels `--midi-device`)
- `--list-audio` — list audio input devices and exit (parallels `--list-midi`)

### AudioConfig Construction from Tuning File

**Gap identified:** `TuningEngine` does not currently expose the `[audio]` section. The raw `AudioSection` is only on `RawTuningFile`, which is consumed during compilation.

**Recommended approach:** Add an `audio_config()` method to `TuningEngine` that returns `Option<AudioConfig>`:

1. During `TuningEngine::compile()`, store the `AudioSection` if present
2. Add `pub fn audio_config(&self) -> Option<AudioConfig>` that converts stored `AudioSection` → `AudioConfig` (filling in `window_size`, `hop_size`, `sample_rate` from defaults)
3. In `main.rs`: `let config = engine.audio_config().unwrap_or_default();`

This keeps the tuning file as the single source of truth for audio parameters (per D-15).

### Signal Level for TUI Meter

**Gap identified:** `AudioMsg` currently only has `NoteOn` and `NoteOff` variants. The signal level meter (D-11) needs RMS level data.

**Recommended approach:** Add `AudioMsg::SignalLevel(f32)` variant to `haydn-audio`:
- Analysis thread computes RMS on each hop (~11ms) and sends `SignalLevel` alongside note events
- Main loop receives `SignalLevel` and updates `TuiState.signal_level: Option<f32>`
- Minimal overhead — one extra f32 per hop, ~90 messages/second

### TUI State Model Changes

```rust
// New enum for input source tracking
pub enum InputSource {
    Midi { velocity: u8 },
    Audio { confidence: f32 },
}

// HistoryEntry changes
pub struct HistoryEntry {
    pub note_name: String,
    pub source: InputSource,    // replaces `velocity: u8`
    pub operation: String,
    pub output_text: Option<String>,
    pub edge_case: Option<String>,
}

// TuiState additions
pub struct TuiState {
    // ... existing fields ...
    pub input_mode: String,        // "MIDI" or "Mic"
    pub signal_level: Option<f32>, // None in MIDI mode, Some(rms) in mic mode
}
```

### Rendering Changes

**Status bar (D-10):** `MIDI: [device]` or `Mic: [device]` — replace raw device name with prefixed version.

**Signal meter (D-11):** Unicode block characters in status bar, only in mic mode:
```
▁▂▃▅▇  ← maps RMS 0.0-1.0 to 5 block levels
```

**History entries (D-12):**
- MIDI: `[C4 v=80] → Push(0)` (existing format)
- Audio: `[C4 ~92%] → Push(0)` (tilde prefix, percentage, replaces velocity)

### Disconnect/Reconnect for Audio (D-16)

Same pattern as MIDI: check if audio stream is still alive, if not pause and wait for device reconnection. The cpal `Stream` object stays alive — but if the device is removed, the stream's error callback fires. Simplest approach: periodic device enumeration check (same as MIDI) + drop/recreate stream on reconnect.

## 4. Required Crate Changes

### haydn-audio (minor additions)
- Add `AudioMsg::SignalLevel(f32)` variant
- Have analysis thread emit signal level on each hop
- These are additive, non-breaking changes

### haydn-tuning (minor addition)
- Store `AudioSection` during `TuningEngine::compile()`
- Add `pub fn audio_config(&self) -> Option<AudioConfig>` method
- Add conversion `AudioSection → AudioConfig` (fill window_size/hop_size/sample_rate from defaults)

### haydn (main integration)
- Add `haydn-audio` dependency to Cargo.toml
- Extend `Cli` struct with `--input`, `--audio-device`, `--list-audio`
- Add audio input path in main function (parallel to MIDI path)
- Enhance `HistoryEntry` with `InputSource` enum
- Enhance `TuiState` with `input_mode`, `signal_level`
- Update status bar, history entry, signal meter rendering
- Add audio disconnect/reconnect handling

### Tuning files (new files)
- `crates/haydn-tuning/tunings/voice.toml` — new file
- `crates/haydn-tuning/tunings/guitar.toml` — new file
- Corresponding `include_str!` and loader functions in `haydn-tuning/src/lib.rs`

## 5. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Voice range too narrow for comfortable programming | Medium | 10 value notes × 19 ops is usable; piano demo needs only ~6 values to produce "Hi!" |
| Audio device enumeration differs per OS | Low | `cpal` + `haydn-audio` abstracts this; same as Phase 6 |
| Signal level meter flicker | Low | Smooth with exponential moving average in TUI update |
| `AudioMsg::SignalLevel` floods channel | Low | One per hop (~90/sec) is trivial overhead for mpsc |
| Guitar tuning parameter values need real-instrument testing | Medium | Use known-good defaults, document as "calibrate to your guitar" |

## Standard Stack
No new dependencies needed. All libraries already in workspace:
- `haydn-audio`: cpal, ringbuf (already present)
- `haydn-tuning`: serde, toml (already present)
- `haydn`: clap, ratatui, crossterm, midir, midly (already present)

## Dont_hand_roll
Nothing to hand-roll. Audio capture exists (Phase 6), tuning engine exists (Phase 3), TUI exists (Phase 5). Phase 7 is pure integration + tuning file authorship.

---

*Research completed: 2026-03-30*
*Phase: 07-audio-integration-instrument-tunings*
