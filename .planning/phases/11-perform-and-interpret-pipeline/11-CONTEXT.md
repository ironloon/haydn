# Phase 11: Perform-and-Interpret Pipeline - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Combined mode where haydn-performer plays a `.ly` score through synthesis (so you hear it) while simultaneously feeding each MIDI note through the haydn tuning engine + VM (so you see it interpreted as a program). One command: hear the music AND watch the computation in real time. This is the "this is what programming sounds like" demo without requiring MIDI hardware.

</domain>

<decisions>
## Implementation Decisions

### Entry Point & CLI Surface
- **D-01:** `--interpret` flag on `haydn-performer`. The performer already owns the `.ly` → `NoteSequence` → synthesis → playback pipeline. Adding `--interpret` taps each note through the tuning engine + VM alongside playback. No new binary needed.
- **D-02:** `--tuning <path>` is **required** when `--interpret` is present. No silent default — the user explicitly chooses which tuning maps notes to VM opcodes. Avoids confusion when VM output doesn't match expectations because of an unexpected tuning file.
- **D-03:** Command example: `haydn-performer score.ly --interpret --tuning piano.toml`
- **D-04:** `haydn-performer` gains `haydn-tuning` and `haydn-vm` as new dependencies. These are lightweight library crates with no audio deps.

### TUI Layout — Dual Display
- **D-05:** **Side-by-side split layout.** Left panel: performer view (current note, score progress, measure tracking). Right panel: VM dashboard (stack, operation history, program output). Both update in real time as each note plays.
- **D-06:** **Dedicated program output panel** on the VM side. Same philosophy as Phase 5 — the program output emerging letter-by-letter is the showpiece moment and deserves its own visual panel, separate from the operation history.
- **D-07:** **Graceful fallback to stacked (top/bottom) layout** when terminal width is under ~100 columns. Usable on smaller terminals without requiring a hard minimum width.

### Synchronization & Timing
- **D-08:** **Lockstep — one note, one VM event.** Each note plays through the synth AND simultaneously feeds the VM. Stack and output update in real time as you hear each note. The dramatic reveal: "Hello World" emerges letter-by-letter synchronized with the music.
- **D-09:** **Loop replay fires during next note/rest duration.** When the VM hits `loop_end`, the replay burst fires rapidly during the following note/rest's audible duration. The VM side shows operations scrolling; the audio side continues the music. Keeps lockstep without introducing artificial pauses.

### Tuning File Selection
- **D-10:** `--tuning` required with `--interpret` (see D-02). Resolution strategy and tuning name display in TUI are agent's discretion.

### Agent's Discretion
- Tuning file resolution strategy (name shortcuts like `--tuning piano` vs full paths only)
- Whether to show the active tuning name in the TUI header/status bar
- Exact panel proportions and border styling within the side-by-side layout
- Narrow terminal breakpoint (recommended ~100 columns)
- How `--quiet` interacts with `--interpret` (likely scrolling text log with interleaved VM events)
- Whether existing performer TUI and VM TUI display code can be reused/composed or need new render functions
- How rests in the score are displayed during lockstep (VM side idle, performer side shows rest)
- Key bindings (q to quit, any toggles)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Performer Implementation (the host binary)
- `crates/haydn-performer/src/main.rs` — Current performer main loop: parse score → synth setup → playback with TUI
- `crates/haydn-performer/src/cli.rs` — CLI arg definitions (add `--interpret` and `--tuning` here)
- `crates/haydn-performer/src/display.rs` — Existing ratatui TUI: scrolling notes, progress bar, measure tracking
- `crates/haydn-performer/src/types.rs` — `NoteEvent`, `ScoreEvent`, `NoteSequence` types
- `crates/haydn-performer/src/parser.rs` — LilyPond parser producing `NoteSequence`
- `crates/haydn-performer/src/synth/` — Synthesis engine (fidelity 0-5, instrument profiles)

### VM Pipeline (the interpretation consumer)
- `crates/haydn/src/lib.rs` — `process_note()`, `process_note_structured()` — the functions that map MIDI → tuning → VM
- `crates/haydn-vm/src/types.rs` — `StepResult`, `Operation`, `Event`, `Opcode` types
- `crates/haydn-vm/src/lib.rs` — `HaydnVm::feed()`, `step()`, `process_event()`, `stack()`, `output()`

### Tuning Engine
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine::map_note(u8) -> Option<Event>`
- `crates/haydn-tuning/tunings/piano.toml` — Default piano tuning

### VM TUI (design reference for the VM panel)
- `crates/haydn/src/display.rs` — Existing VM TUI: stack visualization, operation history, program output panels

### Specification
- `docs/SPECIFICATION.md` §5 — Loop pedal model (recording/replaying states relevant to D-09)

### Prior Phase Decisions
- `.planning/phases/03.1-simulated-performer/03.1-CONTEXT.md` — D-04 through D-07: performer architecture, note event stream API, MIDI interchange format
- `.planning/phases/05-cli-visualization/05-CONTEXT.md` — D-01 through D-07: VM TUI design philosophy, three-panel layout, shared visual language with performer
- `.planning/phases/04-midi-input-pipeline/04-CONTEXT.md` — D-01/D-03: mpsc channel architecture, main loop structure
- `.planning/phases/10-instrument-synthesis-realism/10-CONTEXT.md` — D-05 through D-08: fidelity architecture (0-5 levels)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/haydn/src/lib.rs` — `process_note_structured()` returns `Vec<StepResult>` from a MIDI note — exactly the function needed to feed VM events from performer notes
- `crates/haydn/src/display.rs` — VM TUI rendering code (stack tower, operation history, output panel) — may be extractable or composable into the dual-display
- `crates/haydn-performer/src/display.rs` — Performer TUI rendering code — the left-panel candidate
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine` with `map_note()` — lightweight, no audio deps, easy to add as performer dependency

### Established Patterns
- Both binaries use ratatui + crossterm for TUI rendering with shared visual language (border style, density)
- Both binaries have `--quiet` flag for headless/scripting fallback
- Channel-based architecture (mpsc) for decoupling input from processing
- `NoteEvent.midi_note: u8` is the interchange format between performer and tuning engine

### Integration Points
- `haydn-performer/Cargo.toml` — needs `haydn-tuning` and `haydn-vm` as new dependencies
- `haydn-performer/src/cli.rs` — needs `--interpret` and `--tuning` flags
- `haydn-performer/src/main.rs` — playback loop needs to branch: if `--interpret`, also feed each note through `TuningEngine::map_note()` → `HaydnVm::process_event()`
- `haydn-performer/src/display.rs` — needs dual-panel rendering mode (performer + VM side-by-side)

</code_context>

<specifics>
## Specific Ideas

- The lockstep synchronization means the performer's existing note-by-note playback loop naturally drives the VM — no separate timing mechanism needed. Each note in the `NoteSequence` plays, and during that play call, the note also feeds the VM.
- Loop replay events during the next note duration creates a visually compelling "burst" effect in the VM panel — rapid operations scrolling while the audio sustains. This communicates the loop concept without needing to understand the spec.
- The `.ly` example scores already in `crates/haydn-performer/examples/` (hello-world.ly, fizzbuzz-intro.ly, etc.) serve as natural demo content for this mode.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 11-perform-and-interpret-pipeline*
*Context gathered: 2026-04-01*
