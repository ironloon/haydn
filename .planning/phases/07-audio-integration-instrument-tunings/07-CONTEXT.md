# Phase 7: Audio Integration & Instrument Tunings - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire microphone note events through the full Haydn pipeline (audio → tuning → VM → TUI) and ship default voice and guitar tuning files. Phase 6 delivered the `haydn-audio` crate that produces `AudioMsg` events from microphone input. Phase 7 integrates that into the `haydn` binary — adding `--input mic` as a first-class input mode alongside the existing MIDI path, creating `voice.toml` and `guitar.toml` default tunings, and progressively enhancing the TUI for audio-specific metadata.

This phase does NOT add new audio processing capabilities (Phase 6), new TUI panels (Phase 5), or new CLI commands (Phase 9). It wires existing pieces together and authors the instrument tunings.

</domain>

<decisions>
## Implementation Decisions

### Input Mode Switching
- **D-01:** **`--input midi|mic` enum flag.** New CLI flag selects the input source. `--input midi` is the default (backward compatible with existing behavior). `--input mic` activates microphone capture. The flag is the single point of mode selection — no auto-detection, no ambiguity.
- **D-02:** **Strictly one input at a time.** MIDI and mic cannot run simultaneously. Multi-source input is out of scope (v2). If `--input midi` is active, the microphone is not opened. If `--input mic` is active, MIDI is not connected.
- **D-03:** **Parallel flag family for audio devices.** New flags mirror the existing MIDI pattern:
  - `--audio-device "name"` — select microphone by name substring (parallels `--midi-device`)
  - `--list-audio` — list available audio input devices and exit (parallels `--list-midi`)
  - Existing `--midi-device` and `--list-midi` flags are unchanged and only apply when `--input midi`

### Voice Tuning Design
- **D-04:** **One generic `voice.toml` tuning file.** Ships as a built-in default alongside `piano.toml`. No male/female variants — a single tuning centered on a comfortable unisex range. Users adjust `root_note` to shift the mapping to their personal vocal range.
- **D-05:** **Dense semitone mapping.** Same mapping density as piano — every semitone is meaningful. Voice pitch wobble (~50 cents) means the same sung phrase produces slightly different values each performance. This is intentional: **imprecision is a feature, not a bug.** Each vocal performance is a unique computation.
- **D-06:** **Forgiving `[audio]` defaults for voice.** The voice tuning ships with relaxed parameters (wider `pitch_stability_cents`, lower `confidence_threshold`) so a casual singer gets usable output immediately. The `[audio]` section serves as a **precision dial** — tightening parameters demands operatic precision, loosening them embraces chaos. Well-commented TOML explaining what each knob does to the precision/expressiveness tradeoff.
- **D-07:** **Root note and range.** Generic vocal center (researcher determines exact values — approximately A2-A4 range, ~2 octaves). Value zone occupies the lower portion, operation zone above. Same structural pattern as piano tuning (values below root, operations at/above root) adapted for vocal range.

### Guitar Tuning Design
- **D-08:** **Agent's discretion for guitar tuning specifics.** One `guitar.toml` tuning file following the same principles as voice: dense semitone mapping, imprecision-as-art, well-commented `[audio]` parameters. The researcher/planner will determine:
  - Root note and range appropriate for guitar (standard tuning E2-E6, common playing range ~E2-E5)
  - `[audio]` parameter values calibrated for guitar acoustics (faster onsets than voice, string noise filtering, hammer-on/pull-off speed)
  - Value/operation zone layout adapted for guitar's wider range
  - Inline TOML comments explaining guitar-specific parameter choices

### TUI Progressive Enhancement
- **D-09:** **Three main panels identical in MIDI and mic mode.** Stack, operation history, and program output panels are unchanged. The dashboard layout does not branch on input mode. A performer switching between MIDI keyboard and singing sees the same tool.
- **D-10:** **Input source label adapts.** Status area shows `MIDI: [device name]` or `Mic: [device name]` based on active input mode. Uses the same visual space.
- **D-11:** **Signal level indicator in mic mode.** A small Unicode block meter (`▁▂▃▅▇`) in the status bar showing microphone input level. Helps performers calibrate mic distance/volume. Zero cognitive overhead — glanceable, not distracting. Not shown in MIDI mode.
- **D-12:** **Confidence annotation in operation history.** MIDI history lines show `[C4 v=80] → Push(0)`. Audio history lines show `[C4 ~92%] → Push(0)`. The `~` prefix signals pitch-detected (approximate) vs MIDI (exact). The confidence percentage replaces velocity, occupying the same visual space. Ties into the imprecision-as-art theme — the performer sees the uncertainty.
- **D-13:** **Audio diagnostics hidden by default.** Raw frequency (Hz), noise gate state, algorithm name, and `[audio]` parameter values are not shown in the TUI. Available in `--quiet` mode verbose output or a future debug toggle. The performance dashboard is for performing, not debugging.

### Pipeline Wiring
- **D-14:** **`AudioMsg` converges at `process_note()`.** Phase 6 delivers `start_audio_capture() -> Receiver<AudioMsg>`. The main loop in `haydn/src/main.rs` adds an audio receive path that calls the same `process_note(note, velocity, engine, vm)` function as MIDI. Audio confidence maps to the velocity parameter (or is handled via a wrapper — agent's discretion on exact mapping).
- **D-15:** **Audio config loaded from tuning file `[audio]` section.** The active tuning file's `[audio]` section is parsed into `AudioConfig` and passed to `start_audio_capture()`. If the section is absent, `AudioConfig::default()` is used. The tuning file is the single source of truth for audio parameters.
- **D-16:** **Disconnect/reconnect for audio.** Same pattern as MIDI (Phase 4 D-09/D-10): if the audio device is lost mid-performance, pause and wait for reconnection. VM state preserved. Ctrl+C to exit. Implementation details are agent's discretion.

### Agent's Discretion
- Exact `AudioMsg` → `process_note()` mapping (confidence as velocity, separate field, or wrapper)
- Audio device auto-detect behavior (single device auto-select, same as MIDI pattern from Phase 4 D-06)
- Voice tuning exact root note, value range boundaries, operation assignments
- Guitar tuning all specifics (see D-08)
- `[audio]` parameter values for voice and guitar tunings (researcher should investigate optimal values)
- How `--input` flag interacts with `--demo` mode (demo probably stays MIDI-only, or gains an audio demo — agent decides)
- Quiet mode (`--quiet`) output format for audio events (parallel to existing MIDI quiet format)
- Whether `AudioMsg::NoteOff` needs handling in the main loop (MIDI note-off is logged and discarded per Phase 4 D-16)
- Signal level meter update rate and smoothing in TUI

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Audio Crate (the input source being integrated)
- `crates/haydn-audio/src/lib.rs` — Public API: `start_audio_capture()`, `find_audio_input_device()`, `list_audio_input_devices()`
- `crates/haydn-audio/src/types.rs` — `AudioMsg`, `AudioConfig`, `AudioError` types
- `crates/haydn-audio/src/capture.rs` — Audio capture implementation (ring buffer → analysis thread → mpsc channel)
- `crates/haydn-audio/src/state_machine.rs` — `NoteStateMachine` that produces `AudioMsg` events

### Existing Pipeline (the integration target)
- `crates/haydn/src/main.rs` — Current main event loop: MIDI receive → `process_note()` → TUI render. Phase 7 adds an audio receive path alongside.
- `crates/haydn/src/lib.rs` — `MidiMsg`, `process_note()`, `process_note_structured()`, `format_event_log()` — the convergence point for audio events
- `crates/haydn/src/display.rs` — TUI state model and rendering (Phase 5) — where signal meter and confidence annotation are added

### Tuning System (tuning file authorship)
- `crates/haydn-tuning/src/types.rs` — `RawTuningFile`, `AudioSection`, `Metadata` types — voice/guitar tunings must conform to this schema
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine::map_note(u8)` — the function audio note events call after pitch detection
- `crates/haydn-tuning/tunings/piano.toml` — Reference tuning file. Voice and guitar tunings follow this structure.
- `docs/TUNING.md` — Tuning file format documentation including `[audio]` section reference

### Specification
- `docs/SPECIFICATION.md` §4 — Event interface: `Push(i64)` and `Op(Opcode)`. Audio must produce identical event types as MIDI. Source independence guarantee.
- `docs/SPECIFICATION.md` §6 — Silence and timing: silence is absence of events, no timing semantics.

### Prior Phase Decisions
- `.planning/phases/06-audio-capture-pitch-detection/06-CONTEXT.md` — D-04/D-05: three-thread audio architecture, D-12/D-13/D-14: `AudioMsg` event model and `process_note()` convergence, D-08/D-09: `[audio]` section schema
- `.planning/phases/05-cli-visualization/05-CONTEXT.md` — D-01/D-02/D-03: TUI performance dashboard design, `--quiet` flag, unified visual language
- `.planning/phases/04-midi-input-pipeline/04-CONTEXT.md` — D-01/D-03: mpsc channel main loop pattern, D-06/D-07/D-08: device management pattern (auto-detect/prompt/flag), D-09/D-10/D-11: disconnect/reconnect behavior

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `haydn-audio` crate: Complete audio capture pipeline with `start_audio_capture(device, config) -> Receiver<AudioMsg>`. Ready to wire.
- `haydn/src/lib.rs`: `process_note()` and `process_note_structured()` — the shared convergence point. Audio events call the same functions as MIDI.
- `haydn/src/display.rs`: `TuiState`, `HistoryEntry`, `render_dashboard()` — the TUI state model to enhance with audio metadata.
- `haydn-tuning` crate: `AudioSection` already parsed from `[audio]` TOML. `AudioConfig` construction from tuning file is straightforward.
- `piano.toml`: Reference structure for authoring voice and guitar tunings. Already includes `[audio]` section.

### Established Patterns
- **Device management:** `select_midi_port()` pattern in `main.rs` — auto-detect single, prompt on multiple, `--midi-device` flag. Audio device selection follows the same pattern using `find_audio_input_device()` / `list_audio_input_devices()`.
- **Channel architecture:** MIDI uses `mpsc::channel::<MidiMsg>()`. Audio uses `mpsc::channel` returned by `start_audio_capture()`. Both feed a `recv_timeout` / `try_recv` main loop.
- **TUI state update:** `tui_state.update_from_step(note, velocity, result)` — audio path calls the same method, substituting confidence for velocity.
- **Disconnect/reconnect:** `wait_for_reconnect()` pattern with probe polling. Audio needs equivalent.

### Integration Points
- `main.rs` `Cli` struct: Add `--input`, `--audio-device`, `--list-audio` to clap derive
- `main.rs` main loop: Branch on input mode — MIDI path (existing) or audio path (new), both calling `process_note()`
- `display.rs` `HistoryEntry`: Add field or variant to distinguish MIDI velocity from audio confidence in rendering
- `display.rs` `TuiState`: Add optional signal level field, input mode indicator
- `haydn-tuning/tunings/`: Add `voice.toml` and `guitar.toml` alongside `piano.toml`

</code_context>

<specifics>
## Specific Ideas

- **Imprecision-as-art philosophy:** Voice pitch wobble means no two vocal performances produce identical computation. This is the artistic core of microphone input — the human voice introduces non-determinism through expression. The voice tuning's `[audio]` parameters are the precision dial: loose defaults for casual/chaotic performance, tightenable for trained singers demanding exact programs.
- **Precision dial metaphor:** The `[audio]` section in voice/guitar tunings should have rich inline TOML comments framing parameters as artistic controls, not technical knobs. "Tighten this for operatic precision. Loosen for expressionist chaos."
- **`~` prefix in history lines:** The tilde before confidence percentage (`[C4 ~92%]`) visually communicates "approximately" — the performer sees uncertainty as a first-class concept baked into the display, not hidden.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-audio-integration-instrument-tunings*
*Context gathered: 2026-03-30*
