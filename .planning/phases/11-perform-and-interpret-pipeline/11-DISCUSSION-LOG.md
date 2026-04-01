# Phase 11: Perform-and-Interpret Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 11-perform-and-interpret-pipeline
**Areas discussed:** Entry point & CLI surface, TUI layout — dual display, Synchronization & timing, Tuning file selection

---

## Entry Point & CLI Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Flag on haydn-performer | Add --interpret flag to haydn-performer. Reuses existing .ly parsing, synth, and TUI. | ✓ |
| New binary | New top-level binary (e.g., haydn-play). Clean separation but duplicates .ly parsing and synth setup. | |
| Subcommand on haydn | Add a 'play' subcommand to the haydn VM binary. Keeps the VM as the entry point but requires pulling in performer dependencies. | |

**User's choice:** Flag on haydn-performer (recommended by agent, accepted by user)
**Notes:** User asked for recommendation. Agent recommended flag on performer because it already owns the full .ly → synth → playback pipeline. User confirmed.

### --tuning requirement

| Option | Description | Selected |
|--------|-------------|----------|
| Require --tuning with --interpret | --tuning is required when --interpret is present. No default. | ✓ |
| Default to piano.toml | Default to piano.toml if --tuning not provided. | |
| Auto-detect | Auto-detect from the .ly score content. | |

**User's choice:** Require --tuning (recommended by agent, accepted by user)
**Notes:** User deferred to recommendation.

---

## TUI Layout — Dual Display

| Option | Description | Selected |
|--------|-------------|----------|
| Side-by-side split | Left: performer (notes, progress, measures). Right: VM (stack, ops, output). | ✓ |
| Top/bottom split | Top: performer view. Bottom: VM dashboard. | |
| Performer-primary with VM overlay | Full performer TUI with smaller VM output overlay or footer. | |
| VM-primary with performer status bar | Full VM dashboard with compact performer status line. | |

**User's choice:** Side-by-side split (recommended by agent, accepted by user)

### Program output placement

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated output panel | Program output gets its own sub-panel in the VM side. | ✓ |
| Inline in operation history | Program output appears inline in the operation history. | |
| Full-width footer bar | Program output spans both sides at the bottom. | |

**User's choice:** Dedicated output panel (recommended by agent, accepted by user)

### Narrow terminal handling

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful fallback to stacked layout | Degrade to top/bottom layout under ~100 columns. | ✓ |
| Hard minimum width | Require minimum terminal width, warn and exit if too narrow. | |
| Agent's discretion | Let the agent decide. | |

**User's choice:** Graceful fallback (recommended by agent, accepted by user)

---

## Synchronization & Timing

| Option | Description | Selected |
|--------|-------------|----------|
| Lockstep — one note, one VM event | Each note plays through synth AND feeds the VM simultaneously. Real-time stack/output updates as you hear. | ✓ |
| VM leads, audio follows | VM events fire ahead of audio. Less dramatic. | |
| Instant VM, musical audio | VM processes all notes instantly. Audio plays at musical tempo. | |

**User's choice:** Lockstep (recommended by agent, accepted by user)

### Loop replay timing

| Option | Description | Selected |
|--------|-------------|----------|
| Replay during next note/rest duration | Loop replay events fire rapidly during following note/rest duration. VM shows burst, audio continues. | ✓ |
| Instant replay at loop_end | All replay events fire instantly at loop_end before next note. | |
| Skip loops | Skip loop replay in interpret mode. | |

**User's choice:** Replay during next note/rest duration (recommended by agent, accepted by user)

---

## Tuning File Selection

### Resolution strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Agent's discretion | Let the agent decide resolution strategy. | ✓ |
| Name shortcuts + full paths | Accept shortcuts like --tuning piano. | |
| Full path only | Only accept full file paths. | |

**User's choice:** Agent's discretion

### Tuning name in TUI

| Option | Description | Selected |
|--------|-------------|----------|
| Agent's discretion | Let the agent decide whether to show tuning name. | ✓ |
| Show tuning name in TUI | Display tuning name in header/status bar. | |
| Don't show | Don't show — user already specified on CLI. | |

**User's choice:** Agent's discretion

---

## Agent's Discretion

- Tuning file resolution strategy
- Tuning name visibility in TUI
- Narrow terminal breakpoint
- `--quiet` + `--interpret` interaction
- Panel proportions and border styling
- Rest display during lockstep
- Key bindings

## Deferred Ideas

None — discussion stayed within phase scope.
