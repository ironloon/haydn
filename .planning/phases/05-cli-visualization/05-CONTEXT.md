# Phase 5: CLI Visualization - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Real-time terminal display of VM state so performers can see what their music is computing. The TUI shows stack contents, recent operations, and accumulated program output — updating immediately on each musical event. This phase replaces the throwaway stdout event log from Phase 4 with a proper ratatui-based dashboard that shares visual language with `haydn-performer`. It also adds a `--quiet` flag to preserve the scrolling log for scripting/piping use.

</domain>

<decisions>
## Implementation Decisions

### TUI Vibe & Visual Language
- **D-01:** **Performance dashboard, not debugger.** Clean, glanceable panels designed for a performer who looks up from their instrument. Minimal text, breathing room, prominent information. Not a dense htop-style debugger.
- **D-02:** **Unified visual language with `haydn-performer`.** Same ratatui + crossterm stack, same border style, same visual density and personality. The `haydn` VM TUI and `haydn-performer` playback TUI feel like siblings — both are Haydn tools for musicians. The content differs (stack/operations/output vs score/progress/measures) but the design system is shared.
- **D-03:** **`--quiet` flag for scripting fallback.** When `--quiet` is passed, fall back to the existing scrolling stdout event log (Phase 4's format). TUI is the default mode. This mirrors `haydn-performer`'s `--quiet` flag behavior.

### Stack Visualization
- **D-04:** **Agent's discretion on layout specifics.** Recommended direction: vertical tower (top-of-stack at top), show top N items with overflow indicator, annotate values with ASCII character when in printable range (e.g., `72 'H'`). The performer should be able to anticipate `PrintChar` output by seeing characters on the stack.

### Operation History
- **D-05:** **Scrolling recent history (~8-10 lines), newest at bottom.** Each operation from `StepResult` gets a line in the history panel. Shows enough context to follow the flow ("I pushed 72, then 101, then Add...") without overwhelming. During loop replays (rapid multi-step bursts), history scrolls quickly — this is expected and visually communicates that the loop is executing.

### Output Presentation
- **D-06:** **Dedicated output panel for accumulated program output.** `PrintChar` appends characters, `PrintNum` appends numbers. The output grows over time as the performer "writes" their program. This is the "money moment" — watching "Hello World" emerge letter by letter deserves its own visual spotlight, separate from the operation history.

### Layout Structure
- **D-07:** **Three-panel dashboard.** The TUI has three visible areas: (1) stack visualization, (2) operation history, (3) program output. Exact arrangement (vertical splits, horizontal splits, proportions) is agent's discretion — should feel balanced and glanceable on a standard 80x24 terminal.

### Agent's Discretion
- Loop state indicator (recording/replaying/normal mode) — whether and how to surface this in the TUI
- Edge case display — how `StackUnderflow`, `DivisionByZero` warnings appear (inline in operation history, flash indicator, or separate)
- Exact panel arrangement and proportions
- Stack orientation (vertical tower recommended but not locked)
- Stack depth cap (top-N vs full)
- ASCII annotation formatting
- Color scheme and highlighting choices
- Status bar content (tuning name, device name, session duration, etc.)
- How disconnect/reconnect state appears in the TUI (Phase 4's wait-for-reconnect message)
- Key bindings (q to quit, any debug toggles)
- Whether to extract shared TUI utilities into a common crate or keep display code in each binary

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### VM Interface (data source for the TUI)
- `crates/haydn-vm/src/types.rs` — `StepResult`, `Operation`, `EdgeCase`, `Event`, `Opcode` types — everything the TUI needs to display
- `crates/haydn-vm/src/lib.rs` — `HaydnVm::stack()`, `output()` accessors for current state

### Existing Pipeline (the main loop the TUI integrates into)
- `crates/haydn/src/main.rs` — Current main event loop: `recv → map_note → feed → step → print`. Phase 5 replaces the `print` step with TUI rendering.
- `crates/haydn/src/lib.rs` — `process_note()`, `format_event_log()`, `format_session_summary()` — current stdout formatters that `--quiet` mode preserves

### Performer TUI (visual language reference)
- `crates/haydn-performer/src/display.rs` — Existing ratatui TUI: scrolling note display, progress bar, measure tracking, bordered panels. This is the design reference for the VM TUI.
- `crates/haydn-performer/src/cli.rs` — CLI structure with `--quiet` flag pattern

### Tuning Engine (displayed in TUI)
- `crates/haydn-tuning/src/engine.rs` — `TuningEngine::map_note()` — the mapping step shown in operation history

### Specification
- `docs/SPECIFICATION.md` §5 — Loop pedal model (recording/replaying/normal states) — relevant for loop state indication

### Prior Phase Decisions
- `.planning/phases/04-midi-input-pipeline/04-CONTEXT.md` — D-03: main loop naturally extends for TUI; D-12/D-13: current stdout log is throwaway scaffolding replaced by this phase
- `.planning/phases/02-vm-core/02-CONTEXT.md` — D-01: StepResult struct with operation, stack_snapshot, output, edge_case
- `.planning/phases/03.1-simulated-performer/03.1-CONTEXT.md` — D-17/D-18/D-20: performer TUI design precedent (music-player feel, not VM observation)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `haydn-performer/src/display.rs`: Full ratatui TUI implementation with `run_display()`, crossterm raw mode handling, alternate screen, key event handling. Direct reference for TUI setup and teardown patterns.
- `haydn/src/lib.rs`: `format_event_log()` and `format_session_summary()` — reusable for `--quiet` mode output.
- `haydn/src/lib.rs`: `process_note()` — returns `Vec<String>` of formatted lines; for TUI mode this should return structured data instead (or TUI reads `StepResult` directly).

### Established Patterns
- ratatui + crossterm for terminal UI (proven in `haydn-performer`)
- `clap` for CLI argument parsing (both `haydn` and `haydn-performer`)
- `mpsc` channel for MIDI→main thread communication (Phase 4)
- `ctrlc` crate for Ctrl+C handling (Phase 4)

### Integration Points
- The main event loop in `crates/haydn/src/main.rs` currently prints to stdout after each `process_note()` call. Phase 5 replaces this with TUI rendering while keeping the MIDI receive + tuning map + VM step pipeline unchanged.
- `--quiet` flag added to existing `Cli` struct in `main.rs`
- TUI must coexist with the disconnect/reconnect flow (Phase 4 D-09/D-10)

</code_context>

<specifics>
## Specific Ideas

- The performer envisions a cohesive Haydn tool family — `haydn` and `haydn-performer` should look and feel like they belong together. Shared visual personality, not just shared dependencies.
- The "Hello World emerging letter by letter" moment is the flagship demo visual. The output panel should make this feel special.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-cli-visualization*
*Context gathered: 2026-03-29*
