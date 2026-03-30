---
phase: 05-cli-visualization
plan: 02
status: complete
started: 2026-03-30
completed: 2026-03-30
---

## Summary

Wired the TUI display into the main event loop with dual-mode operation. TUI mode (default) uses a non-blocking unified event loop: crossterm keyboard polling, `try_recv()` MIDI drain, frame-budget rendering at ~30fps. `--quiet` flag preserves the exact Phase 4 scrolling text log behavior. Added `process_note_structured()` returning raw `Vec<StepResult>` for TUI consumption. MIDI disconnect/reconnect shows in TUI status bar. Terminal restored on quit and on panic via custom panic hook.

## Key Files

### Modified
- `crates/haydn/src/main.rs` — Dual-mode event loop (TUI default, `--quiet` for log), panic hook for terminal restore, `--quiet` CLI flag
- `crates/haydn/src/lib.rs` — Added `process_note_structured()` function

## Verification
- `cargo build -p haydn` — compiles warning-free
- `cargo test -p haydn` — 17 pipeline tests pass
- Manual testing requires MIDI device (deferred to Plan 05-03)

## Deviations
None.
