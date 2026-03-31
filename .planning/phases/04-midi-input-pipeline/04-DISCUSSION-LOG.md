# Phase 4: MIDI Input Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 04-midi-input-pipeline
**Areas discussed:** Pipeline threading, Device management, Live output, Velocity & note-off handling

---

## Pipeline Threading

### Threading Model

| Option | Description | Selected |
|--------|-------------|----------|
| mpsc channel | MIDI callback sends raw notes over std mpsc channel, main thread processes | ✓ |
| Direct mutex | Callback acquires mutex on tuning+VM, processes inline | |
| Lock-free ring buffer | ringbuf crate, zero allocation, polling complexity | |

**User's choice:** mpsc channel (accepted recommendation)
**Notes:** User asked for full pros/cons breakdown. Latency overhead (~0.1ms) negligible against <5ms budget. Clean separation, easy to test, naturally extends for Phase 5 TUI.

### Crate Placement

| Option | Description | Selected |
|--------|-------------|----------|
| In `haydn` binary crate | Binary wires MIDI + tuning + VM; simplest for v1 | ✓ |
| New `haydn-midi` crate | Clean testable boundary, reusable | |

**User's choice:** `haydn` binary crate (accepted recommendation)
**Notes:** Only one consumer for v1. Phase 5 TUI lives in same binary, shares main loop.

---

## Device Management

### Discovery & Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-detect single, prompt on multiple | One device → use it. Multiple → numbered list + prompt. --midi-device flag. | ✓ |
| Always auto-detect first available | Grab first device, --midi-device to override | |
| Always list and prompt | Even with one device, confirm selection | |

**User's choice:** Auto-detect single, prompt on multiple (accepted recommendation)
**Notes:** Matches standard MIDI software behavior. --midi-device flag for scripting.

### Disconnect Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful exit | Print message, flush output, exit | |
| Wait for reconnect | Pause session, preserve VM state, resume on reconnect | ✓ |
| Hot-plug recovery | Auto-detect disconnection, poll for reconnection, auto-resume | |

**User's choice:** Wait for reconnect (accepted recommendation)
**Notes:** Live performance tool — losing VM state is unacceptable. User showed mock UX message. Ctrl+C to quit during wait.

---

## Live Output

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal event log | Scrolling stdout: note → mapping → VM result per line | ✓ |
| Output only | Only PrintChar/PrintNum results visible | |
| Compact status line | Single overwritten line with stack/last op/output | |

**User's choice:** Minimal event log (accepted recommendation)
**Notes:** First playable experience needs full visibility for debugging tuning mappings. Replaced by Phase 5 TUI.

---

## Velocity & Note-Off Handling

### Velocity

| Option | Description | Selected |
|--------|-------------|----------|
| Ignore for v1 | All note-on treated equally | |
| Pass through but don't use | Capture and log velocity, tuning engine ignores it | ✓ |
| Full velocity support | Extend map_note API, velocity-based triggers | |

**User's choice:** Pass through but don't use (accepted recommendation)
**Notes:** User asked whether deferring full velocity support would complicate adding it later. Answer: no — data already flows through pipeline, promoting it is a mechanical API change (add parameter to map_note, add velocity trigger type). TOML schema extensible by design.

### Note-Off

| Option | Description | Selected |
|--------|-------------|----------|
| Silently consume | Receive and discard | |
| Log but discard | Show in event log for debugging, no VM event | ✓ |

**User's choice:** Log but discard (accepted recommendation)

### Sustained/Held Notes

| Option | Description | Selected |
|--------|-------------|----------|
| No repeat events | Held key = one note-on, release = one note-off | |
| Not applicable for MIDI | MIDI keyboards don't re-trigger on hold; only matters for audio (Phase 6) | ✓ |

**User's choice:** Not applicable for MIDI (accepted recommendation)

---

## Agent's Discretion

- Exact MIDI message parsing approach
- Event log formatting details
- Reconnection polling interval
- CLI argument structure
- Error message wording
- MIDI channel filtering (likely not needed)

## Deferred Ideas

- Velocity-based tuning triggers (future tuning enhancement)
- MIDI channel filtering (--midi-channel flag)
- MIDI CC/pitch bend mapping to VM operations
