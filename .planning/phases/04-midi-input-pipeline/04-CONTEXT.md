# Phase 4: MIDI Input Pipeline - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

First end-to-end playable experience — capture real-time MIDI note events from hardware, map them through the tuning engine into VM events, and execute them live. The performer connects a MIDI keyboard, plays notes, and sees computation happen in real time. This phase delivers the pipeline glue in the `haydn` binary crate: MIDI capture → tuning mapping → VM execution → stdout event log.

</domain>

<decisions>
## Implementation Decisions

### Pipeline Threading
- **D-01:** **mpsc channel architecture.** The `midir` callback sends raw MIDI note-on messages over a standard `std::sync::mpsc` channel. The main thread receives from the channel, maps through `TuningEngine::map_note()`, feeds `HaydnVm::feed()` + `step()`, and prints results. Clean separation of MIDI thread and processing thread.
- **D-02:** No mutexes, no lock-free ring buffers. Channel overhead (~0.1ms) is negligible against the <5ms latency target. The VM processes events in microseconds.
- **D-03:** The main loop is: `loop { recv → map_note → feed → step → print }`. This naturally extends for Phase 5 (TUI reads the same StepResults).

### Crate Placement
- **D-04:** All pipeline code lives in the **`haydn` binary crate** (`crates/haydn/`). The binary is the orchestrator that wires MIDI + tuning + VM together. No new crate needed for v1 — there's only one consumer.
- **D-05:** `haydn` binary gains `midir` as a dependency. `haydn-vm` and `haydn-tuning` remain zero-audio-dep library crates.

### Device Management
- **D-06:** **Auto-detect single device, prompt on multiple.** If exactly one MIDI input port is available, connect to it silently. If multiple ports are found, print a numbered list and prompt the user to select. Zero-device case prints an error with troubleshooting guidance.
- **D-07:** **`--midi-device` CLI flag** bypasses the prompt — matches by name substring. For scripting and power users.
- **D-08:** Device listing: `haydn --list-midi` shows available MIDI input ports and exits. Useful for troubleshooting.

### Disconnect & Reconnect
- **D-09:** **Wait for reconnect on disconnect.** When the MIDI device disconnects mid-performance, the session pauses — print a message ("MIDI device disconnected. Waiting for device... Ctrl+C to quit"), preserve all VM state (stack, memory, loop recordings), and poll for reconnection.
- **D-10:** On reconnect, resume the session seamlessly. The performer's computation state is intact — they pick up where they left off.
- **D-11:** Ctrl+C is the escape hatch during the wait. Graceful shutdown: flush any pending VM output, print final stack state, exit zero.

### Live Output (Pre-TUI)
- **D-12:** **Minimal scrolling event log to stdout.** Each MIDI event prints one line showing: the note played, what the tuning engine mapped it to, and the VM result. Example: `[C4] → Push(0)  |  Stack: [0]` or `[E4] → Op(Add)  |  Stack: [5]  |  Out: "H"`.
- **D-13:** This is throwaway scaffolding — replaced entirely by Phase 5's real TUI. It just needs to work and be readable, not look polished.
- **D-14:** VM `PrintChar`/`PrintNum` output gets special treatment — clearly visible in the log so the performer sees their program's output emerge.

### Velocity & Note-Off Handling
- **D-15:** **Velocity captured and logged, not used by tuning engine.** MIDI note-on velocity (0-127) is included in the channel message and displayed in the event log for debugging, but `TuningEngine::map_note(u8)` API is unchanged. Velocity-based tuning triggers are a future extension — the data flows through the pipeline now so promoting it later is mechanical (add a parameter to `map_note`, add a velocity trigger type to the rule engine).
- **D-16:** **Note-off events logged but discarded.** Note-off messages appear in the event log ("C4 released") for debugging visibility but do not produce VM events. Per spec D-11, silence/release is a no-op at the VM level.
- **D-17:** **Sustained/held notes not applicable for MIDI.** MIDI keyboards produce exactly one note-on per key press. Held keys do not re-trigger. This only matters for audio input (Phase 6) where continuous signal requires onset detection.

### Agent's Discretion
- Exact MIDI message parsing approach (raw bytes vs `midir` helpers)
- Event log formatting details (colors, alignment, timestamp inclusion)
- Reconnection polling interval and detection mechanism
- CLI argument structure within `clap` (subcommands vs flags)
- Error message wording for zero-device and connection failure cases
- Whether to support MIDI channel filtering (likely not needed for v1)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### VM Interface (the pipeline's consumer)
- `docs/SPECIFICATION.md` §4 — Event types: `Push(i64)` and `Op(Opcode)`, metadata (ignored in v1)
- `docs/SPECIFICATION.md` §5 — Loop pedal model: Recording/Replaying/Normal states
- `crates/haydn-vm/src/types.rs` — `Event`, `Opcode`, `StepResult`, `EdgeCase` types
- `crates/haydn-vm/src/lib.rs` — `HaydnVm::feed()`, `step()`, `provide_input()` API

### Tuning Engine (the pipeline's mapper)
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine::map_note(u8) -> Option<Event>` — the function the MIDI pipeline calls
- `crates/haydn-tuning/src/types.rs` — `RawTuningFile`, `Metadata` types
- `crates/haydn-tuning/tunings/piano.toml` — Default piano tuning (the tuning file used during first playable demo)

### MIDI Library (already proven in workspace)
- `midir` 0.10 — Already a dependency in `haydn-performer`. Known to work on this project for MIDI I/O.

### Prior Phase Decisions
- `.planning/phases/01-language-specification/01-CONTEXT.md` — D-04/D-05: event interface contract (`Push`/`Op`), tuning owns all mapping
- `.planning/phases/01-language-specification/01-CONTEXT.md` — D-11: silence is a no-op at VM level
- `.planning/phases/02-vm-core/02-CONTEXT.md` — D-01/D-02: step-by-step VM with StepResult, process_event convenience wrapper
- `.planning/phases/02-vm-core/02-CONTEXT.md` — D-10: event-driven streaming architecture (no "load program" step)
- `.planning/phases/03-tuning-system/03-CONTEXT.md` — D-01/D-02/D-03: rule-based engine with exact note, range, and interval triggers
- `.planning/phases/03-tuning-system/03-CONTEXT.md` — D-11/D-12/D-13: piano tuning root=C4(60), value zone 36-59, white-key opcodes

### Project Context
- `.planning/PROJECT.md` — Core vision, constraints, key decisions
- `.planning/REQUIREMENTS.md` — MIDI-01, MIDI-02, MIDI-03, MIDI-04 requirements for this phase

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/haydn-tuning/src/engine.rs`: `TuningEngine::compile()` and `map_note(u8) -> Option<Event>` — the tuning engine is ready to consume MIDI note numbers
- `crates/haydn-vm/src/lib.rs`: `HaydnVm::feed(Event)` + `step() -> Option<StepResult>` — the VM is ready to execute events
- `crates/haydn-performer/src/midi.rs`: `midir` usage patterns — MIDI output proven in the workspace, input follows the same library
- `crates/haydn-performer/src/display.rs`: `midi_to_name(u8) -> String` — converts MIDI note numbers to human-readable names (e.g., 60 → "C4"), reusable for the event log

### Established Patterns
- Cargo workspace with crate-per-concern: `haydn-vm`, `haydn-tuning`, `haydn-performer`, `haydn`
- Event-driven architecture: events flow via `feed()`, results via `step()` returning `StepResult`
- `midir` for MIDI I/O, `clap` for CLI argument parsing (both proven in `haydn-performer`)

### Integration Points
- The `haydn` binary crate currently has minimal code (`crates/haydn/src/main.rs`). This phase adds the MIDI pipeline as its primary functionality.
- `haydn` depends on `haydn-vm` and `haydn-tuning` (library crates). Adds `midir` for MIDI input.
- The pipeline flow: `midir` callback → `mpsc::Sender` → main loop → `TuningEngine::map_note()` → `HaydnVm::feed()` → `step()` → print `StepResult`

</code_context>

<specifics>
## Specific Ideas

- This is the **first playable experience** — the moment the project goes from "library that processes events" to "plug in a keyboard and compute." The event log should feel responsive and immediate, even if it's ugly.
- The wait-for-reconnect behavior reflects that this is a live performance tool — losing VM state mid-performance would be like a DAW crashing when you unplug a cable. The show must go on.
- The minimal event log doubles as a tuning debugging tool — "I played C4 and got Push(0), then E4 and got Op(Add)" tells the performer exactly how their tuning file maps notes to computation.

</specifics>

<deferred>
## Deferred Ideas

- **Velocity-based tuning triggers** — Pass velocity to `map_note()` and add velocity range triggers in the rule engine. Data already flows through the pipeline; promoting it is a mechanical API change. Belongs in a future tuning enhancement phase.
- **MIDI channel filtering** — Some setups send on multiple channels. Could add `--midi-channel` flag. Not needed for v1 single-instrument use.
- **MIDI CC/pitch bend support** — Control Change and pitch bend could map to VM operations (e.g., mod wheel = modifier). New capability, not v1.

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-midi-input-pipeline*
*Context gathered: 2026-03-29*
