# Phase 3: Tuning System — Research

**Researched:** 2026-03-29
**Discovery Level:** 1 (Quick Verification — well-known Rust crates, schema design already done)
**Status:** Complete

## Research Question

"What do I need to know to PLAN the tuning system implementation well?"

The tuning system bridges musical input and the VM. It parses TOML tuning files into an in-memory rule engine, validates them, and maps MIDI note numbers to `Event::Push(i64)` and `Event::Op(Opcode)` values. The schema design was completed during `/gsd-discuss-phase` (Option A: Rules + Shorthands). This research covers implementation specifics: crate structure, serde deserialization patterns, rule engine internals, validation architecture, default piano tuning design, and testing strategy.

## 1. Crate Structure Decision

### Recommendation: New `haydn-tuning` crate in workspace

**Why not inside `haydn-vm`?**
- `haydn-vm` has zero external dependencies — this is a deliberate design virtue
- Adding `toml`, `serde`, `thiserror` to `haydn-vm` pollutes the pure computation layer
- The tuning system is a conceptually separate concern: parsing config → producing events

**Workspace structure after Phase 3:**
```
crates/
  haydn/           # Binary crate (main.rs)
  haydn-vm/        # VM library (zero deps, pure computation)
  haydn-tuning/    # Tuning engine (depends on haydn-vm for types + toml/serde)
```

**Dependency graph:**
```
haydn-tuning  →  haydn-vm   (for Event, Opcode types)
haydn-tuning  →  toml       (TOML parsing)
haydn-tuning  →  serde      (deserialization)
haydn-tuning  →  thiserror  (error types)
haydn (binary) → haydn-vm + haydn-tuning
```

## 2. Dependencies

### Confirmed Versions (crates.io, 2026-03-29)

| Crate | Version | MSRV | Purpose |
|-------|---------|------|---------|
| `toml` | 1.1.0 | 1.85 | TOML deserialization via serde |
| `serde` | 1.0.228 | 1.56 | Derive-based deserialization |
| `thiserror` | 2.0.18 | 1.68 | Structured error types |

All MIT/Apache-2.0 dual licensed — compatible with Haydn's MIT license.

**Note:** `toml` 1.1.0 MSRV 1.85 matches the project target (Rust stable 1.85+). Use `serde = { version = "1", features = ["derive"] }` for `#[derive(Deserialize)]`.

### Cargo.toml for haydn-tuning

```toml
[package]
name = "haydn-tuning"
version = "0.0.1"
edition = "2021"
description = "Tuning file parser and rule engine for the Haydn esoteric programming language"
license = "MIT"
repository = "https://github.com/jwgeller/haydn"

[dependencies]
haydn-vm = { path = "../haydn-vm" }
serde = { version = "1", features = ["derive"] }
toml = "1.1"
thiserror = "2"
```

## 3. TOML Schema → Rust Types

### 3.1 Deserialization Model

The TOML file deserializes into a `RawTuningFile` struct, then gets compiled into an optimized `TuningEngine` for runtime matching. Two-phase design: parse → compile.

```rust
// Phase 1: Deserialization target (mirrors TOML structure exactly)
#[derive(Debug, Deserialize)]
pub struct RawTuningFile {
    pub metadata: Metadata,
    #[serde(default)]
    pub values: Option<ValuesSection>,
    #[serde(default)]
    pub operations: Option<BTreeMap<u8, String>>,  // note_number → opcode_name
    #[serde(default)]
    pub rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub root_note: u8,           // MIDI number (0-127)
    pub instrument: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub partial_coverage: bool,  // D-09: suppress missing-opcode warnings
}

#[derive(Debug, Deserialize)]
pub struct ValuesSection {
    pub range: [u8; 2],  // [from, to] inclusive MIDI note range
}

#[derive(Debug, Deserialize)]
pub struct RawRule {
    pub trigger: RawTrigger,
    pub emit: RawEmit,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RawTrigger {
    #[serde(rename = "note")]
    Note { note: u8 },
    #[serde(rename = "range")]
    Range { from: u8, to: u8 },
    #[serde(rename = "interval")]
    Interval { semitones: i8 },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RawEmit {
    #[serde(rename = "push")]
    Push { value: i64 },
    #[serde(rename = "push_offset")]
    PushOffset,  // semitone offset from root_note
    #[serde(rename = "op")]
    Op { opcode: String },
}
```

**Key serde patterns:**
- `#[serde(tag = "type")]` for internally-tagged enums — matches `{ type = "note", note = 60 }` TOML syntax
- `BTreeMap<u8, String>` for `[operations]` section — TOML integer keys deserialize to `u8`, values are opcode name strings
- `Option<T>` + `#[serde(default)]` for optional sections

### 3.2 Compiled Rule Engine

```rust
// Phase 2: Compiled for fast matching at runtime
pub struct TuningEngine {
    rules: Vec<CompiledRule>,
    root_note: u8,
    metadata: Metadata,
    last_note: Option<u8>,  // for interval trigger tracking
}

pub struct CompiledRule {
    trigger: Trigger,
    action: Action,
}

pub enum Trigger {
    ExactNote(u8),
    NoteRange { from: u8, to: u8 },
    Interval(i8),  // semitone distance from last note
}

pub enum Action {
    Push(i64),
    PushOffset { root_note: u8 },  // compute offset at match time
    Op(Opcode),
}
```

**Compilation step:**
1. Expand `[values]` shorthand → one `NoteRange` rule with `PushOffset` action
2. Expand `[operations]` table → one `ExactNote` rule per entry with `Op(opcode)` action
3. Parse opcode strings → `Opcode` enum variants (with typo suggestions on failure)
4. Append raw `[[rules]]` as-is
5. Validate: no overlapping triggers (D-10)

**Matching algorithm:** Linear scan of rules, first match wins. For v1 with <100 rules this is O(n) per event — trivially fast. No need for hash maps or binary search.

### 3.3 Opcode String Parsing

Need a function to convert TOML opcode strings to `Opcode` enum:

```rust
fn parse_opcode(name: &str) -> Result<Opcode, TuningError> {
    match name {
        "dup" => Ok(Opcode::Dup),
        "swap" => Ok(Opcode::Swap),
        // ... all 19
        _ => Err(TuningError::UnknownOpcode {
            found: name.to_string(),
            suggestion: suggest_opcode(name),
        }),
    }
}
```

This requires `Opcode` to be importable from `haydn-vm`. Currently `Opcode` is `pub` — good.

**Typo suggestion:** Use Levenshtein distance or simple substring matching against the 19 known opcode names. For a set this small, a hand-rolled approach is fine — no external crate needed.

## 4. Validation Architecture

### 4.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum TuningError {
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("in [{section}]: {message}")]
    Validation { section: String, message: String },

    #[error("unknown opcode '{found}'{}", suggestion.as_ref().map(|s| format!(" — did you mean '{s}'?")).unwrap_or_default())]
    UnknownOpcode { found: String, suggestion: Option<String> },

    #[error("overlapping rules: {rule_a} and {rule_b} both match note {note}")]
    Overlap { rule_a: String, rule_b: String, note: u8 },

    #[error("note {note} out of MIDI range (0-127)")]
    NoteOutOfRange { note: u8 },
}
```

### 4.2 Validation Pass (D-08: collect all errors in one pass)

The validator runs AFTER deserialization succeeds (valid TOML syntax) and BEFORE compilation:

1. **Metadata validation:** `root_note` in 0-127, `name` non-empty
2. **Values section:** `range[0] <= range[1]`, both in 0-127
3. **Operations table:** each key in 0-127, each value is a valid opcode name
4. **Raw rules:** each trigger has valid MIDI numbers, each emit has valid opcode names
5. **Overlap detection (D-10):** expand all rules to the set of notes they match, check for duplicates
6. **Coverage check (D-07, D-09):** if `partial_coverage` is false, warn about any of the 19 opcodes not mapped

Returns `Vec<TuningError>` — ALL issues, not just the first one.

### 4.3 Warning vs Error Distinction

- **Errors** (prevent loading): TOML syntax errors, unknown opcodes, overlapping rules, out-of-range notes
- **Warnings** (informational): missing opcode coverage (unless `partial_coverage = true`)

```rust
pub struct ValidationResult {
    pub errors: Vec<TuningError>,
    pub warnings: Vec<String>,
}
```

### 4.4 Overlap Detection Algorithm

For a small rule set (<100 rules), brute force is fine:

1. For each `ExactNote(n)` rule, record `n → rule_index`
2. For each `NoteRange { from, to }` rule, record `from..=to → rule_index`
3. For each note 0-127 that appears in >1 rule → report overlap error
4. `Interval` rules don't overlap with note rules (different trigger type)
5. Two `Interval` rules with the same semitone distance → overlap error

## 5. Default Piano Tuning Design

### 5.1 Layout Overview (D-11 through D-15)

```
MIDI 36-59 (C2-B3)  →  VALUE ZONE: Push(semitone_offset_from_root)
                        C2=Push(-24) ... B3=Push(-1)

MIDI 60+ white keys  →  OPERATION ZONE: 19 opcodes on C major scale notes
                        Black keys: agent's discretion
```

### 5.2 White Key → Opcode Assignment Strategy

**Constraint:** 19 opcodes on white keys from C4 upward.

White keys C4 through G6:
```
C4(60) D4(62) E4(64) F4(65) G4(67) A4(69) B4(71)    ← Octave 4: 7 keys
C5(72) D5(74) E5(76) F5(77) G5(79) A5(81) B5(83)    ← Octave 5: 7 keys
C6(84) D6(86) E6(88) F6(89) G6(91)                    ← Octave 6: 5 keys
                                                        Total: 19 keys ✓
```

**Musicality principle (D-14):** Common instruction sequences should produce pleasant melodic fragments in C major. The assignment groups related operations by register and places frequently-paired operations on adjacent or consonant-interval keys.

**Proposed assignment (subject to planning review):**

| Register | Keys | Operations | Rationale |
|----------|------|-----------|-----------|
| Octave 4 (low) | C4-B4 | Stack + Arithmetic | Most frequent operations live in comfortable middle range. Push→Add→Sub progressions create stepwise motion. |
| Octave 5 (mid) | C5-B5 | Comparison + Control + I/O | Loop constructs at the pivot point. Print ops near end of phrases. |
| Octave 6 (high) | C6-G6 | Memory + Utility | Less frequent operations in upper register. |

**Specific mapping (draft — optimized for BF-translated Hello World patterns):**

```
C4(60) = add        ← Root note position: most frequent arithmetic op
D4(62) = sub        ← Step up from add: the "inverse pair"
E4(64) = dup        ← Major third: prepare-for-operation is common
F4(65) = drop       ← Half step from dup (discard after use)
G4(67) = mul        ← Perfect fifth: strong position for multiplication
A4(69) = div        ← Natural pair with mul
B4(71) = mod        ← Leading tone, completes arithmetic set

C5(72) = loop_start ← Octave of root: "entering the loop" = new beginning
D5(74) = loop_end   ← Step up: loop pair close together for fast playing
E5(76) = eq         ← Comparison after loop check
F5(77) = gt
G5(79) = lt         ← Comparisons cluster mid-register
A5(81) = print_char ← Output as "resolution" before line ending
B5(83) = print_num  ← Leading tone: "statement delivery"

C6(84) = read       ← High register: input ops (less frequent)
D6(86) = store
E6(88) = load       ← Memory operations paired
F6(89) = swap       ← Stack manipulation in upper register
G6(91) = rotate     ← Least frequent stack op, highest position
```

**Why this sounds good for Hello World:**
- Push values (low register C2-B3) → add/sub (C4/D4) creates ascending motion
- Loop patterns: dup(E4) → loop_start(C5) is a rising octave — dramatic!
- loop_end(D5) → add(C4) is descending — resolution back to computation
- Heavy print_char(A5) sequences create a recognizable melodic pattern

**Black keys (agent's discretion):** Unmapped in default tuning. Can be used for constants (Push(0), Push(1), Push(10)) in custom tunings.

### 5.3 piano.toml File

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

## 6. Public API Design

### 6.1 Core API

```rust
// Load and validate a tuning file from TOML string
pub fn load_tuning(toml_content: &str) -> Result<TuningEngine, Vec<TuningError>>;

// Load from file path
pub fn load_tuning_file(path: &Path) -> Result<TuningEngine, Vec<TuningError>>;

// Get the default piano tuning (embedded)
pub fn default_piano_tuning() -> TuningEngine;
```

### 6.2 TuningEngine API

```rust
impl TuningEngine {
    /// Map a MIDI note-on event to a VM Event.
    /// Returns None if no rule matches this note.
    pub fn map_note(&mut self, midi_note: u8) -> Option<Event>;

    /// Get metadata (name, instrument, etc.)
    pub fn metadata(&self) -> &Metadata;

    /// Validate coverage and return warnings
    pub fn coverage_warnings(&self) -> Vec<String>;

    /// Reset interval tracking state (e.g., between performances)
    pub fn reset(&mut self);
}
```

**Note:** `map_note` takes `&mut self` because interval tracking requires remembering the last note played.

### 6.3 Default Piano Embedding

The default piano tuning TOML is embedded in the binary via `include_str!`:

```rust
const DEFAULT_PIANO_TOML: &str = include_str!("../tunings/piano.toml");

pub fn default_piano_tuning() -> TuningEngine {
    load_tuning(DEFAULT_PIANO_TOML)
        .expect("built-in piano tuning must be valid")
}
```

The `tunings/piano.toml` file lives in the `haydn-tuning` crate directory.

## 7. Testing Strategy

### 7.1 Unit Tests (in haydn-tuning)

| Area | Tests |
|------|-------|
| TOML parsing | Valid file round-trips, missing sections, malformed TOML |
| Opcode parsing | All 19 opcodes, unknown opcode with typo suggestion |
| Validation | Overlapping rules, out-of-range notes, missing coverage, all-errors-collected |
| Rule matching | Exact note, note range, interval triggers, first-match-wins |
| Default piano | Loads successfully, all 19 opcodes mapped, value range correct |
| Edge cases | Empty tuning file, note 0, note 127, negative intervals |

### 7.2 Integration Tests

- Load `piano.toml` → map a sequence of notes → verify correct Events produced
- BF Hello World note sequence through piano tuning → feed to VM → verify "Hello World!\n" output
- Invalid tuning file → collect all errors → verify error messages contain section/location info

### 7.3 Test Helper

```rust
/// Create a minimal tuning engine for testing
pub fn test_tuning(ops: &[(u8, Opcode)]) -> TuningEngine { ... }
```

## 8. Documentation Plan (DOC-07)

The tuning documentation should include:
1. **Format reference:** All TOML sections, fields, types, defaults
2. **Tutorial:** Step-by-step guide to creating a custom tuning file
3. **Examples:** Piano (default), minimal (just a few ops), interval-based
4. **Opcode reference table:** All 19 opcodes with their names for use in tuning files
5. **Validation error catalog:** What each error means and how to fix it

Target location: `docs/TUNING.md` — standalone document referenced from main README.

## 9. Pitfalls & Gotchas

| Pitfall | Mitigation |
|---------|-----------|
| TOML integer keys in `[operations]` | `BTreeMap<u8, String>` handles this — TOML integers deserialize to Rust integers. However, TOML keys are always strings in the TOML spec, so `[operations]` with integer-looking keys like `60 = "add"` works because toml-rs parses them as string keys. Use `BTreeMap<String, String>` and parse the keys as u8 manually, OR use TOML dotted keys. **Actually: bare integer keys in TOML are valid as table keys — toml 1.x handles `BTreeMap<u8, String>` via serde.** Verify in testing. |
| Interval tracking across rests | `last_note` should NOT be cleared on unmapped notes. A rest is simply "no event" — the last played note persists for interval calculation. |
| `Opcode` is in `haydn-vm` but tuning needs it | `haydn-vm` already exports `Opcode` and `Event` as `pub` — tuning crate `use haydn_vm::{Event, Opcode};` |
| Thread safety of TuningEngine | `TuningEngine` has `&mut self` on `map_note` due to `last_note` tracking. This is fine for single-threaded use. For the multi-threaded architecture (Phase 4+), the MIDI/audio thread will own the `TuningEngine` instance. |
| TOML key ordering | TOML tables are unordered by spec. Rule evaluation order is: all shorthand rules first (values range, then operations in note-number order), then raw rules in array order. Document this clearly. |

## 10. What NOT to Build

- **Runtime hot-swapping** — Deferred to Phase 4/9
- **Velocity-based triggers** — Deferred; D-02 specifies only 3 trigger types for v1
- **Sequence-based triggers** — Deferred
- **Audio-specific tunings** — Phase 7
- **Crate publishing / namespace reservation** — Phase 9

---
*Phase: 03-tuning-system*
*Researched: 2026-03-29*
