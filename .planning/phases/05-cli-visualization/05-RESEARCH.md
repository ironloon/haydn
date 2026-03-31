# Phase 5: CLI Visualization — Research

## Discovery Level: 1 (Quick Verification)

The stack is well-known: ratatui + crossterm, proven in `haydn-performer`. Research focuses on integration patterns with the existing MIDI event loop and shared design system.

## Current Architecture Analysis

### Main Event Loop (crates/haydn/src/main.rs)

The current loop is a blocking `recv_timeout(500ms)` on `mpsc::Receiver<MidiMsg>`:

```
while running {
    match rx.recv_timeout(500ms) {
        Ok(NoteOn) → process_note() → println!()
        Ok(NoteOff) → println!()
        Timeout → check disconnect → maybe reconnect
        Disconnected → break
    }
}
```

**Problem for TUI:** This loop blocks for up to 500ms at a time. A TUI needs ~30fps rendering (33ms per frame) and crossterm keyboard event polling. The current blocking pattern won't work.

### Performer TUI Pattern (crates/haydn-performer/src/main.rs + display.rs)

The performer solves this differently because its data flow is one-directional (time-based playback):
- Audio plays in a `rodio::Sink` (fire-and-forget)
- TUI runs in a **separate thread** via `std::thread::spawn`
- TUI polls `crossterm::event::poll(33ms)` for keyboard input
- TUI calculates current position from `Instant::now() - start`
- `stop_signal: Arc<AtomicBool>` coordinates shutdown

**Key insight:** The performer's TUI doesn't need to receive events from the main thread — it derives state from elapsed time. The Haydn VM TUI is different: it must react to MIDI events that mutate VM state.

## Architecture Decision: Event Loop Integration

### Option A: TUI in Separate Thread (Like Performer)

```
Main thread: recv MIDI → process → send TuiState to display thread
Display thread: poll keyboard + render at 30fps
Channel: mpsc::Sender<TuiState> from main to display
```

Pros: Clean separation, main thread stays simple
Cons: Need to serialize all VM state into a `TuiState` struct on every event, extra channel overhead, two threads touching terminal (risky with crossterm)

### Option B: Unified Main Loop (Recommended) ✓

```
Main thread: loop {
    poll crossterm events (non-blocking, 0ms)
    try_recv MIDI (non-blocking)
    if event received → process → mark dirty
    if dirty or 33ms elapsed → render frame
    sleep remaining frame budget
}
```

Pros: Simple, no shared state, VM + TUI in same thread, no synchronization
Cons: Slight coupling between TUI and event processing

**Recommendation: Option B.** The performer used Option A because audio playback is inherently threaded (rodio). For MIDI input → VM → render, everything can stay single-threaded. The `try_recv()` pattern (non-blocking) replaces `recv_timeout()`.

### Revised Main Loop Architecture

```rust
// Pseudocode for TUI mode main loop
let mut last_render = Instant::now();
let frame_budget = Duration::from_millis(33); // ~30fps

while running.load(Ordering::Relaxed) {
    // 1. Handle keyboard events (non-blocking)
    if crossterm::event::poll(Duration::ZERO)? {
        if let Event::Key(key) = crossterm::event::read()? {
            if key.code == KeyCode::Char('q') { break; }
        }
    }

    // 2. Drain MIDI events (non-blocking, process all pending)
    let mut dirty = false;
    loop {
        match rx.try_recv() {
            Ok(MidiMsg::NoteOn { note, velocity }) => {
                // process through tuning + VM, push to history
                dirty = true;
            }
            Ok(MidiMsg::NoteOff { note }) => { /* update state */ }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => { /* shutdown */ }
        }
    }

    // 3. Check disconnect on timeout cadence (every 500ms, not every frame)
    if last_disconnect_check.elapsed() > Duration::from_millis(500) { ... }

    // 4. Render if dirty or frame budget elapsed
    if dirty || last_render.elapsed() >= frame_budget {
        terminal.draw(|frame| render_dashboard(frame, &tui_state))?;
        last_render = Instant::now();
    }

    // 5. Brief sleep to avoid busy-waiting
    std::thread::sleep(Duration::from_millis(1));
}
```

## Shared Visual Language with haydn-performer

### Current Performer TUI Style
- `Block::default().title(" haydn-performer ").borders(Borders::ALL).border_type(BorderType::Rounded)`
- Cyan color for progress gauge
- Clean layout: vertical chunks with fixed heights
- ratatui 0.29 + crossterm 0.28

### Design System to Share
Both TUIs should use:
- `BorderType::Rounded` for all panel borders
- Title format: `" haydn "` / `" haydn-performer "` (space-padded, lowercase)
- Consistent color palette (Cyan for primary accents)
- Same margin (1) and clean spacing
- Similar information density — glanceable, not dense

### Extract Shared Crate?
**No — keep display code in each binary.** The layouts differ entirely (performer: playback timeline; haydn: VM state dashboard). Shared elements are just style constants (border type, colors) which are trivial to replicate. A shared crate adds dependency weight for minimal benefit. If a third TUI tool appears, reconsider then.

## TUI State Model

The TUI needs to track:

```rust
struct TuiState {
    // Stack panel
    stack: Vec<i64>,              // from vm.stack()

    // Operation history panel
    history: Vec<HistoryEntry>,   // rolling buffer, ~last 20 entries
    
    // Output panel
    output: Vec<u8>,              // from vm.output()
    
    // Status bar
    tuning_name: String,
    device_name: String,
    connected: bool,
    loop_state: String,           // "Normal" / "Recording" / "Replaying"
    
    // Internal
    dirty: bool,
}

struct HistoryEntry {
    note_name: String,
    velocity: u8,
    operation: String,            // formatted from Operation enum
    edge_case: Option<String>,
}
```

**Data sourcing:**
- Stack: `vm.stack()` (already public `&[i64]`)
- Output: `vm.output()` (already public `&[u8]`)
- Operation history: built from `StepResult` returned by `vm.process_event()`
- Loop state: `vm.loop_state` is `pub(crate)` — need to expose a public accessor or derive from `Operation` variants

### Loop State Exposure

Current: `pub(crate) loop_state: LoopState` in `HaydnVm`

Options:
1. Add `pub fn loop_state(&self) -> &str` on HaydnVm (returns "Normal"/"Recording"/"Replaying")
2. Infer from last `Operation` (LoopEntered=Recording, LoopReplaying=Replaying, LoopExited/EndOfBuffer=Normal)

**Recommendation: Option 1.** Simple accessor, no inference bugs. Add to haydn-vm/src/lib.rs.

## Layout Design

### Three-Panel Dashboard (D-07)

Terminal 80x24 minimum:

```
╭─ haydn ───────────────────────────────────────────────────────────────────────╮
│                                                                               │
│  ╭─ Stack ──────────╮  ╭─ Operations ──────────────────────────────────────╮  │
│  │   72 'H'         │  │  [C4 v=80] → Push(72)                            │  │
│  │  101 'e'         │  │  [E4 v=90] → Push(101)                           │  │
│  │  108 'l'         │  │  [G4 v=85] → Push(108)                           │  │
│  │  108 'l'         │  │  [A4 v=75] → Push(108)                           │  │
│  │  111 'o'         │  │  [B4 v=70] → Push(111)                           │  │
│  │                  │  │  [C5 v=88] → Op(PrintChar)  → 'H'                │  │
│  │                  │  │  [D5 v=82] → Op(PrintChar)  → 'e'                │  │
│  │                  │  │  [E5 v=79] → Op(PrintChar)  → 'l'                │  │
│  │  ──── 5 items ── │  │                                                   │  │
│  ╰──────────────────╯  ╰──────────────────────────────────────────────────╯  │
│                                                                               │
│  ╭─ Output ───────────────────────────────────────────────────────────────╮  │
│  │  Hello                                                                  │  │
│  ╰────────────────────────────────────────────────────────────────────────╯  │
│                                                                               │
│  Piano (root=C4)  │  USB MIDI Keyboard  │  Normal  │  q: quit               │
╰───────────────────────────────────────────────────────────────────────────────╯
```

Layout structure:
```rust
// Outer: vertical split
Constraint::Min(0)     // Main content area
Constraint::Length(1)  // Status bar

// Main content: horizontal split
Constraint::Percentage(25)  // Stack panel (left)
Constraint::Percentage(75)  // Right side

// Right side: vertical split
Constraint::Percentage(70)  // Operation history
Constraint::Percentage(30)  // Output panel
```

### Stack Panel (D-04)
- Vertical tower, top-of-stack at top
- Show top N items (fit to panel height)
- ASCII annotation: `72 'H'` when 32 ≤ value ≤ 126
- Overflow indicator: `── N items ──` at bottom when stack is deeper than visible

### Operation History (D-05)
- Scrolling list, newest at bottom
- ~8-10 visible lines
- Format: `[NoteName v=vel] → Operation` with optional output/edge case
- During loop replays: lines scroll quickly (expected)

### Output Panel (D-06)
- Accumulated `PrintChar`/`PrintNum` output
- Wraps at panel width
- The "money moment" — Hello World emerging letter by letter

### Status Bar
- Tuning name | Device name | Loop state | Key bindings
- Shows "⚠ Disconnected" when MIDI device lost

## Dependencies to Add

To `crates/haydn/Cargo.toml`:
```toml
ratatui = "0.29"
crossterm = "0.28"
```

Match performer's versions for consistency. No new unknown dependencies.

## Key Integration Points

1. **`--quiet` flag**: Add to existing `Cli` struct. When set, skip TUI setup, use existing `println!()` path (current behavior).
2. **TUI setup/teardown**: `enable_raw_mode()`, `EnterAlternateScreen` before loop; `disable_raw_mode()`, `LeaveAlternateScreen` after. Same pattern as performer.
3. **Disconnect in TUI mode**: Show status in TUI status bar instead of `eprintln!()`. The reconnect logic stays the same but feedback channel changes.
4. **Session summary**: On quit, restore terminal and print session summary (existing `format_session_summary()`).
5. **process_note() return value**: Currently returns `Vec<String>` of formatted log lines. For TUI mode, need `Vec<StepResult>` instead. Either: add a new function returning structured data, or have TUI mode call `engine.map_note()` + `vm.process_event()` directly (simpler).

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| crossterm + MIDI callback thread conflict | Low | MIDI callback writes to mpsc channel only — doesn't touch terminal |
| TUI rendering blocks MIDI processing | Low | 33ms frame budget is tiny; MIDI events queue in mpsc channel |
| Terminal resize during session | Medium | ratatui handles resize events automatically via crossterm |
| Disconnect messages in raw mode | Medium | Route all output through TUI panels, never println/eprintln in TUI mode |

## Validation Architecture

### Critical Paths to Test
1. **Stack display accuracy**: VM stack matches TUI display after each event
2. **Operation history correctness**: Each StepResult produces correct history entry
3. **Output accumulation**: PrintChar/PrintNum output matches vm.output()
4. **`--quiet` mode**: Existing behavior unchanged (regression test)
5. **TUI startup/teardown**: Terminal restored on normal quit and on panic

### Testable Boundaries
- `TuiState` construction from VM state (unit testable, no terminal needed)
- `HistoryEntry` formatting from `StepResult` (unit testable)
- Stack display formatting with ASCII annotations (unit testable)
- `--quiet` CLI flag behavior (integration testable via cargo test)

### Not Easily Testable
- Actual TUI rendering (requires terminal — manual verification)
- Keyboard event handling in TUI mode
- Disconnect/reconnect display updates

---

*Research completed: 2026-03-29*
*Discovery level: 1 — Quick verification of known stack*
