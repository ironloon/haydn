# Phase 7: Audio Integration & Instrument Tunings - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 07-audio-integration-instrument-tunings
**Areas discussed:** Input mode switching, Voice tuning design, Guitar tuning design, TUI adaptations for audio

---

## Input Mode Switching

| Option | Description | Selected |
|--------|-------------|----------|
| `--input midi\|mic` flag | New enum flag, default midi, parallel `--audio-device`/`--list-audio` flags | ✓ |
| Positional subcommands | `haydn midi` / `haydn mic` as subcommands with own flags | |
| Auto-detect | No flag, detect available devices automatically, `--input` to override | |

**User's choice:** `--input midi|mic` flag
**Notes:** Recommended because the operation is the same verb (perform), only the source differs. Flags are idiomatic for parameters, subcommands for different verbs. Auto-detect rejected as hostile — performer must know which source is active. Strictly one-at-a-time (no simultaneous MIDI + mic).

---

## Voice Tuning Design

### Vocal range approach

| Option | Description | Selected |
|--------|-------------|----------|
| One generic voice.toml | Single tuning targeting ~2 octave unisex range, `root_note` as shift knob | ✓ |
| Separate male/female tunings | Different root notes and ranges per voice type | |

**User's choice:** One generic `voice.toml`
**Notes:** User can copy and adjust `root_note` to personalize — tunability without shipping confusing variants.

### Mapping density

| Option | Description | Selected |
|--------|-------------|----------|
| Dense semitone mapping | Every semitone meaningful, wobble produces variation | ✓ |
| Wide intervals (whole tones/pentatonic) | Fewer notes, more reliable hit rate, but absorbs wobble | |

**User's choice:** Dense semitone mapping
**Notes:** User explicitly values imprecision-as-art — "weird output because art, no performance is the same." Dense mapping means voice wobble produces different values each time, making every performance unique. Wide mapping would absorb the wobble and kill the non-determinism.

### Precision dial

| Option | Description | Selected |
|--------|-------------|----------|
| `[audio]` params as artist control surface | Forgiving defaults, tightenable for trained singers | ✓ |
| Fixed parameters | Ship one set of calibrated values, not user-adjustable | |

**User's choice:** `[audio]` params as precision dial
**Notes:** User emphasized tunability and "fuzzy interpretation." Loose defaults = casual/chaotic. Tight params = operatic precision demanded. The tuning file comments should frame these as artistic controls.

### Musicality

**User's choice:** Yes, musicality matters — common programs should produce singable patterns
**Notes:** Same principle as piano tuning (common sequences produce pleasant melodic fragments).

---

## Guitar Tuning Design

| Option | Description | Selected |
|--------|-------------|----------|
| Agent's discretion | Same principles as voice, adapted for guitar acoustics | ✓ |
| User-directed design | Detailed user input on guitar-specific choices | |

**User's choice:** Agent's discretion
**Notes:** User deferred all guitar specifics. Researcher/planner will design root note, range, `[audio]` parameters, and value/operation zones following the same dense-mapping, imprecision-as-art, well-commented philosophy.

---

## TUI Adaptations for Audio

| Option | Description | Selected |
|--------|-------------|----------|
| Progressive enhancement | Same 3 panels, add signal meter + confidence annotation in status/history | ✓ |
| Identical TUI | No changes, just device name differs | |
| Audio debug panel | New panel showing pitch detection internals | |

**User's choice:** Progressive enhancement
**Notes:** User liked the idea of enriching the existing UI without adding new panels. Specifics locked:
- Input source label: `Mic: [name]` vs `MIDI: [name]`
- Signal level meter: Unicode block chars in status bar (mic mode only)
- Confidence in history: `[C4 ~92%]` replaces `[C4 v=80]` — tilde communicates approximation
- Raw diagnostics hidden by default (future debug toggle)

---

## Agent's Discretion

- Guitar tuning all specifics (root note, range, audio params, zone layout)
- Voice tuning exact root note and range boundaries
- `AudioMsg` → `process_note()` mapping details
- Audio device auto-detect behavior
- `--demo` interaction with `--input mic`
- Audio disconnect/reconnect implementation
- Signal meter update rate and smoothing

## Deferred Ideas

None — discussion stayed within phase scope.
