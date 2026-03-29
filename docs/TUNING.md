# Tuning File Format

Tuning files map musical input (MIDI notes) to Haydn VM instructions.
Each note you play triggers either a `Push` (put a number on the stack) or an `Op` (execute an opcode).
Tuning files use [TOML](https://toml.io/) syntax and have the `.toml` extension.

See [SPECIFICATION.md](SPECIFICATION.md) for the full Haydn instruction set and VM behavior.

## File Structure

A tuning file has four sections. Only `[metadata]` is required:

```toml
[metadata]        # Required: identifies the tuning
[values]          # Optional: shorthand for push-value note range
[operations]      # Optional: shorthand for note → opcode mappings
[[rules]]         # Optional: custom rules for advanced mappings
```

**Evaluation order:** When a MIDI note arrives, the engine checks rules in this order:

1. **Values range** — if the note falls within `[values].range`, push its offset from root
2. **Operations** — if the note matches an exact key in `[operations]`, execute the opcode
3. **Raw rules** — check `[[rules]]` in array order

First match wins. If nothing matches, the note is ignored.

## Section Reference

### `[metadata]` — Tuning Identity (required)

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | yes | — | Human-readable tuning name |
| `root_note` | u8 | yes | — | MIDI note number (0–127). Semitone offsets are calculated from this note. |
| `instrument` | String | yes | — | Instrument identifier (e.g., "piano", "guitar", "synth") |
| `author` | String | no | — | Tuning author's name |
| `partial_coverage` | bool | no | `false` | If `true`, suppresses warnings when not all 19 opcodes are mapped |

Example:

```toml
[metadata]
name = "My Custom Tuning"
root_note = 60
instrument = "piano"
author = "J. S. Bach"
partial_coverage = false
```

### `[values]` — Push-Value Zone (optional)

Maps a range of MIDI notes to push their semitone offset from `root_note`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `range` | [u8, u8] | yes (within section) | Inclusive MIDI note range `[low, high]` |

Each note in the range produces `Push(note - root_note)`. For example, with `root_note = 60`:

- MIDI 48 (C3) → `Push(-12)` (one octave below)
- MIDI 60 (C4) → `Push(0)` (root)
- MIDI 55 (G3) → `Push(-5)` (a fourth below)

```toml
[values]
range = [36, 59]    # C2 through B3: two octaves of push values
```

### `[operations]` — Note-to-Opcode Mapping (optional)

Maps individual MIDI notes to opcodes. Keys are MIDI note numbers, values are opcode names.

```toml
[operations]
60 = "add"
62 = "sub"
64 = "dup"
65 = "drop"
```

All 19 valid opcode names are listed in the [Opcode Reference](#opcode-reference) below.

### `[[rules]]` — Custom Rules (optional)

An array of rules for advanced mappings. Each rule has a `trigger` and an `emit`.

#### Trigger Types

**`note`** — matches a specific MIDI note:

```toml
[[rules]]
[rules.trigger]
type = "note"
note = 84

[rules.emit]
type = "push"
value = 0
```

**`range`** — matches any note in an inclusive range:

```toml
[[rules]]
[rules.trigger]
type = "range"
from = 96
to = 108

[rules.emit]
type = "push_offset"
```

**`interval`** — matches when the semitone distance from the previous note equals the given value:

```toml
[[rules]]
[rules.trigger]
type = "interval"
semitones = 7

[rules.emit]
type = "op"
opcode = "add"
```

Interval triggers enable musical patterns: a perfect fifth (7 semitones) triggers `add`, a perfect fourth (5 semitones) triggers `sub`, etc.

#### Emit Types

| Type | Fields | Description |
|------|--------|-------------|
| `push` | `value: i64` | Push a literal value onto the stack |
| `push_offset` | (none) | Push `note - root_note` (same as values zone) |
| `op` | `opcode: String` | Execute the named opcode |

## Opcode Reference

All 19 opcodes in the Haydn instruction set. Stack notation: `(before -- after)`.

| Name | Description | Stack Effect |
|------|-------------|--------------|
| `dup` | Duplicate top value | `(a -- a a)` |
| `swap` | Swap top two values | `(a b -- b a)` |
| `drop` | Remove top value | `(a b -- a)` |
| `rotate` | Rotate top three values | `(a b c -- b c a)` |
| `add` | Add top two values | `(a b -- a+b)` |
| `sub` | Subtract top from second | `(a b -- a-b)` |
| `mul` | Multiply top two values | `(a b -- a*b)` |
| `div` | Integer divide second by top | `(a b -- a/b)` |
| `mod` | Modulo second by top | `(a b -- a%b)` |
| `eq` | Push 1 if equal, 0 otherwise | `(a b -- a==b)` |
| `gt` | Push 1 if second > top, 0 otherwise | `(a b -- a>b)` |
| `lt` | Push 1 if second < top, 0 otherwise | `(a b -- a<b)` |
| `loop_start` | Begin loop (skip body if top is 0) | `(n -- n)` |
| `loop_end` | End loop (replay body if top ≠ 0) | `(n -- n)` |
| `print_num` | Print top as decimal number | `(a -- )` |
| `print_char` | Print top as ASCII character | `(a -- )` |
| `read` | Read one byte from input | `( -- byte)` |
| `store` | Store value at address in memory | `(addr val -- )` |
| `load` | Load value from address in memory | `(addr -- val)` |

## Default Piano Tuning

The built-in piano tuning maps a standard MIDI keyboard:

```toml
[metadata]
name = "Default Piano"
root_note = 60
instrument = "piano"

[values]
range = [36, 59]

[operations]
60 = "add"
62 = "sub"
64 = "dup"
65 = "drop"
67 = "mul"
69 = "div"
71 = "mod"
72 = "loop_start"
74 = "loop_end"
76 = "eq"
77 = "gt"
79 = "lt"
81 = "print_char"
83 = "print_num"
84 = "read"
86 = "store"
88 = "load"
89 = "swap"
91 = "rotate"
```

### Layout

**Value zone** (MIDI 36–59, C2–B3): Two octaves of push values. Each note pushes its semitone offset from middle C. This gives a range of −24 to −1 for numeric input.

**Operation zone** (white keys C4–G6): Each white key maps to one opcode:

| MIDI | Note | Opcode | MIDI | Note | Opcode |
|------|------|--------|------|------|--------|
| 60 | C4 | `add` | 77 | F5 | `gt` |
| 62 | D4 | `sub` | 79 | G5 | `lt` |
| 64 | E4 | `dup` | 81 | A5 | `print_char` |
| 65 | F4 | `drop` | 83 | B5 | `print_num` |
| 67 | G4 | `mul` | 84 | C6 | `read` |
| 69 | A4 | `div` | 86 | D6 | `store` |
| 71 | B4 | `mod` | 88 | E6 | `load` |
| 72 | C5 | `loop_start` | 89 | F6 | `swap` |
| 74 | D5 | `loop_end` | 91 | G6 | `rotate` |
| 76 | E5 | `eq` | | | |

**Black keys** are unmapped in the default tuning — available for custom rules if you extend it.

## Creating a Custom Tuning

### Step 1: Start with Metadata

Pick a root note and name your tuning:

```toml
[metadata]
name = "Minimal Arithmetic"
root_note = 48
instrument = "keyboard"
partial_coverage = true    # We're only mapping a few opcodes
```

### Step 2: Define a Value Zone

Choose a MIDI range for pushing numbers onto the stack:

```toml
[values]
range = [24, 47]    # Two octaves below root
```

Notes 24–47 will push values from −24 to −1, relative to root note 48.

### Step 3: Map Opcodes

Assign notes to the opcodes you need:

```toml
[operations]
48 = "add"
50 = "sub"
52 = "mul"
53 = "div"
55 = "dup"
57 = "drop"
59 = "print_num"
```

### Step 4: Add Custom Rules (optional)

Use raw rules for advanced triggers like intervals:

```toml
[[rules]]
[rules.trigger]
type = "interval"
semitones = 12     # Octave jump

[rules.emit]
type = "op"
opcode = "loop_start"
```

### Step 5: Validate

Run your tuning through the validator:

```sh
haydn --validate-tuning my-tuning.toml
```

The validator checks all errors in a single pass and reports them together.

### Complete Example: Minimal Arithmetic

```toml
[metadata]
name = "Minimal Arithmetic"
root_note = 48
instrument = "keyboard"
partial_coverage = true

[values]
range = [24, 47]

[operations]
48 = "add"
50 = "sub"
52 = "mul"
53 = "div"
55 = "dup"
57 = "drop"
59 = "print_num"
```

## Example Tunings

### Interval-Based Tuning

Uses musical intervals between consecutive notes as triggers. The distance between notes determines the operation:

```toml
[metadata]
name = "Interval Explorer"
root_note = 60
instrument = "any"
partial_coverage = true

[values]
range = [36, 59]

[[rules]]
[rules.trigger]
type = "interval"
semitones = 7      # Perfect fifth

[rules.emit]
type = "op"
opcode = "add"

[[rules]]
[rules.trigger]
type = "interval"
semitones = 5      # Perfect fourth

[rules.emit]
type = "op"
opcode = "sub"

[[rules]]
[rules.trigger]
type = "interval"
semitones = 4      # Major third

[rules.emit]
type = "op"
opcode = "mul"

[[rules]]
[rules.trigger]
type = "interval"
semitones = 3      # Minor third

[rules.emit]
type = "op"
opcode = "div"

[[rules]]
[rules.trigger]
type = "interval"
semitones = 12     # Octave

[rules.emit]
type = "op"
opcode = "print_num"
```

### Chromatic Tuning

Every semitone in one octave maps to an opcode. Compact but less musical:

```toml
[metadata]
name = "Chromatic C4"
root_note = 60
instrument = "any"
partial_coverage = true

[values]
range = [36, 59]

[operations]
60 = "dup"
61 = "swap"
62 = "drop"
63 = "rotate"
64 = "add"
65 = "sub"
66 = "mul"
67 = "div"
68 = "mod"
69 = "eq"
70 = "gt"
71 = "lt"
72 = "loop_start"
73 = "loop_end"
74 = "print_num"
75 = "print_char"
76 = "read"
77 = "store"
78 = "load"
```

All 19 opcodes fit in a single octave (C4–F#5). Every key is used — no gaps, but harder to play musically.

## Validation Errors

The validator checks all errors in a single pass. Here are the most common messages:

| Error | Cause | Fix |
|-------|-------|-----|
| `unknown opcode 'dupe' — did you mean 'dup'?` | Typo in opcode name | Check spelling against the [opcode reference](#opcode-reference) |
| `overlapping rules: operations[60] and rules[0] both match note 60` | Two rules trigger on the same note | Remove one or change the note assignment |
| `note 200 out of MIDI range (0-127)` | Invalid MIDI number | Use values 0–127 |
| `in [values]: range[0] (80) must be <= range[1] (40)` | Inverted range bounds | Swap the two numbers |
| `TOML parse error: ...` | Syntax error in TOML | Check brackets, quotes, and indentation |
| `unmapped opcodes: loop_start, loop_end, ...` | Not all 19 opcodes are mapped | Map missing opcodes, or set `partial_coverage = true` |

The `partial_coverage` flag is useful when you intentionally only need a subset of opcodes.
Set it to `true` to suppress the missing-opcode warnings.

## Tips

- Keep the value zone at least 2 octaves wide for a useful numeric range (−24 to −1 covers most needs)
- Place frequently-used opcodes on comfortable, easy-to-reach keys
- The default piano tuning is a good starting template — copy and modify it
- Black keys make great targets for custom rules extended from the default tuning
- Use interval triggers to create musically expressive programs — the code sounds like music
