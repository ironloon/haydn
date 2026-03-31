# Phase 5: CLI Visualization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 05-cli-visualization
**Areas discussed:** TUI vibe & layout, Stack visualization, Operation history, Output presentation

---

## TUI Vibe & Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Performance dashboard | Clean, glanceable panels. Guitar-tuner-meets-terminal feel. Minimal text, large type, breathing room. | ✓ |
| Live debugger | Dense, information-rich. Scrolling event history, full stack, memory map, loop state. Like htop for a VM. | |
| Hybrid with toggle | Default performer-friendly, Tab/d reveals debug panel. Best of both. | |

**User's choice:** Performance dashboard — unified visual language with `haydn-performer`
**Notes:** User specifically wanted to "match vision/choices between the modules/packages" — the TUI should feel like a sibling of `haydn-performer`, not a separate tool. Also confirmed `--quiet` flag for scripting fallback, mirroring performer's pattern.

---

## Stack Visualization

| Option | Description | Selected |
|--------|-------------|----------|
| Vertical tower, top-N, ASCII annotated | Top-of-stack at top, show last N items with overflow, annotate printable values with ASCII character | ✓ |
| Horizontal row (matching performer's note display) | Left-to-right, consistent with performer's `→` separator pattern | |
| Full stack (no cap) | Show every item, scrollable | |

**User's choice:** "revisit later, but just go with your recommendation" — recorded as agent's discretion with recommended direction.
**Notes:** User acknowledged this is a real decision but was comfortable deferring to agent judgment.

---

## Operation History

| Option | Description | Selected |
|--------|-------------|----------|
| Last operation only | Single line showing most recent event. Simple but no trail. | |
| Scrolling recent history (~8-10 lines) | Last N operations, newest at bottom. Enough context to follow flow without overwhelming. | ✓ |
| Full scrolling log | Everything since session start. Comprehensive but cluttered for live performance. | |

**User's choice:** "okay, go for your recommendation" — scrolling recent history, ~8-10 lines.
**Notes:** None.

---

## Output Presentation

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated output panel | Bordered section accumulating PrintChar/PrintNum output. Grows over time. Clear separation from operation history. | ✓ |
| Inline with operation history | Output events appear in scrolling history with special formatting. Simpler layout. | |
| Both (panel + inline markers) | Dedicated panel shows accumulated output AND history entries for output ops get highlighted. | |

**User's choice:** "okay go for your recommendation" — dedicated output panel.
**Notes:** None.

---

## Agent's Discretion

- Stack layout specifics (orientation, depth cap, annotation format)
- Loop state indicator (recording/replaying/normal)
- Edge case display approach
- Panel arrangement and proportions
- Color scheme
- Status bar content
- Disconnect/reconnect state in TUI
- Key bindings

## Deferred Ideas

None — discussion stayed within phase scope.
