# Requirements: Haydn

**Defined:** 2026-03-28
**Core Value:** The live performance IS the program — music becomes computation in real time

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Language Specification

- [ ] **SPEC-01**: Formal language specification covering all instructions, execution model, and I/O behavior with no ambiguity
- [ ] **SPEC-02**: All edge cases explicitly specified (stack underflow, division by zero, invalid memory access, empty stack operations, pitch detection failure behavior)
- [ ] **SPEC-03**: Turing completeness proof via reduction from a known TC language or UTM implementation
- [ ] **SPEC-04**: Jump/loop semantics defined for live-input context (how branching works when the program is generated in real time)
- [ ] **SPEC-05**: Note-as-value semantics specified (how notes map to numeric values on the stack)
- [ ] **SPEC-06**: Interval-as-operation semantics specified (how intervals between consecutive notes trigger operations)
- [ ] **SPEC-07**: Silence/rest behavior defined (what happens when no input is detected)
- [ ] **SPEC-08**: Sustained note behavior defined (held notes vs repeated notes)

### Virtual Machine

- [x] **VM-01**: Stack-based virtual machine executes all Haydn instructions correctly *(Phase 2)*
- [x] **VM-02**: Stack operations: push, dup, swap, drop, rotate *(Phase 2)*
- [x] **VM-03**: Arithmetic operations: add, sub, mul, div, mod *(Phase 2)*
- [x] **VM-04**: Comparison operations: eq, gt, lt *(Phase 2)*
- [x] **VM-05**: Control flow: conditional jump, unconditional jump, loop *(Phase 2)*
- [x] **VM-06**: I/O operations: print number, print character, read input *(Phase 2)*
- [x] **VM-07**: Memory operations: store to address, load from address *(Phase 2)*
- [x] **VM-08**: VM handles all specified edge cases gracefully per the language spec *(Phase 2)*

### Tuning System

- [x] **TUNE-01**: Tuning file format documented and parseable (TOML-based configuration)
- [x] **TUNE-02**: Tuning files map musical gestures (notes, intervals, pitch ranges) to VM opcodes
- [x] **TUNE-03**: Default piano tuning ships with the binary (works out of the box with MIDI keyboard)
- [ ] **TUNE-04**: Default voice tuning ships with the binary (works with microphone singing)
- [ ] **TUNE-05**: Default guitar tuning ships with the binary (works with microphone guitar)
- [x] **TUNE-06**: User can create custom tuning files for any instrument or style
- [x] **TUNE-07**: Tuning file validation with clear error messages on misconfiguration

### MIDI Input

- [x] **MIDI-01**: Real-time MIDI input captures note-on/note-off events from any connected MIDI device
- [x] **MIDI-02**: MIDI note events map to language operations via the active tuning file
- [x] **MIDI-03**: MIDI input latency is imperceptible during live performance (<5ms)
- [x] **MIDI-04**: Graceful handling of MIDI device connection/disconnection

### Audio Input

- [ ] **AUD-01**: Real-time microphone input captures audio from system default or user-selected audio device
- [ ] **AUD-02**: Pitch detection converts continuous audio signal to discrete note events (monophonic)
- [ ] **AUD-03**: Onset detection distinguishes new notes from sustained sound
- [ ] **AUD-04**: Noise gating prevents ambient noise from triggering spurious operations
- [ ] **AUD-05**: Audio-to-note events map to language operations via the active tuning file
- [ ] **AUD-06**: Audio input pipeline latency is acceptable for live performance (<60ms end-to-end)

### CLI Visualization

- [ ] **VIS-01**: Terminal UI displays current stack contents in real time during performance
- [ ] **VIS-02**: Terminal UI displays the last operation executed
- [ ] **VIS-03**: Terminal UI displays program output (printed numbers/characters)
- [ ] **VIS-04**: Terminal UI updates immediately on each musical event
- [ ] **VIS-05**: Visualization does not interfere with audio processing performance

### Documentation & Content

- [ ] **DOC-01**: Hello World example program with tuning file and performance instructions
- [ ] **DOC-02**: FizzBuzz example program performable on at least one instrument
- [ ] **DOC-03**: Cat program (echo input) example
- [ ] **DOC-04**: Truth Machine example program
- [ ] **DOC-05**: README in esolang tradition — fun, irreverent, technically precise, with compelling examples
- [ ] **DOC-06**: Language specification published as a standalone document
- [x] **DOC-07**: Tuning file format documentation with examples for creating custom tunings
- [ ] **DOC-08**: Esolang wiki page with spec summary, instruction table, examples, and interpreter link

### Distribution

- [ ] **DIST-01**: Installable via `cargo install haydn`
- [ ] **DIST-02**: Binary runs on macOS, Linux, and Windows
- [ ] **DIST-03**: CLI interface: `haydn --input midi` or `haydn --input mic` to start a session
- [ ] **DIST-04**: CLI supports selecting tuning file: `haydn --tuning piano.toml`
- [ ] **DIST-05**: CLI supports listing available MIDI devices and audio inputs

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Web & Distribution

- **WEB-01**: Browser-based interpreter via WASM with Web MIDI and WebAudio support
- **WEB-02**: Online playground where visitors can try Haydn without installing

### Advanced Input

- **ADV-01**: Chord recognition from MIDI input (simultaneous notes as compound operations)
- **ADV-02**: Multi-performer / ensemble mode with networked VM synchronization

### Content

- **CONT-01**: Performance demo videos showing real musicians programming in Haydn
- **CONT-02**: Self-interpreter (a Haydn program that interprets Haydn)
- **CONT-03**: Additional tuning files for drums, wind instruments, exotic instruments

### Recording

- **REC-01**: Serialize performance events for replay
- **REC-02**: Export performance as event log for analysis

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Audio output / sonification | Haydn's concept is music→computation, not computation→music. Sonic Pi already does that. |
| GUI / rich TUI visualization | CLI matches esolang tradition. GUI adds massive dependency weight. |
| Tempo / rhythm awareness | Note timing as semantic meaning adds enormous complexity. Haydn responds to WHAT you play, not HOW FAST. |
| IDE / editor support | There is no text source code. The performance IS the program. |
| Package manager / module system | Over-engineering. Tuning files are simple configs shareable as files. |
| General-purpose language features | Esoteric means constrained. Adding string ops, file I/O, networking destroys what makes esolangs interesting. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SPEC-01 | Phase 1 | Pending |
| SPEC-02 | Phase 1 | Pending |
| SPEC-03 | Phase 1 | Pending |
| SPEC-04 | Phase 1 | Pending |
| SPEC-05 | Phase 1 | Pending |
| SPEC-06 | Phase 1 | Pending |
| SPEC-07 | Phase 1 | Pending |
| SPEC-08 | Phase 1 | Pending |
| VM-01 | Phase 2 | Pending |
| VM-02 | Phase 2 | Pending |
| VM-03 | Phase 2 | Pending |
| VM-04 | Phase 2 | Pending |
| VM-05 | Phase 2 | Pending |
| VM-06 | Phase 2 | Pending |
| VM-07 | Phase 2 | Pending |
| VM-08 | Phase 2 | Pending |
| TUNE-01 | Phase 3 | Complete |
| TUNE-02 | Phase 3 | Complete |
| TUNE-03 | Phase 3 | Complete |
| TUNE-04 | Phase 7 | Pending |
| TUNE-05 | Phase 7 | Pending |
| TUNE-06 | Phase 3 | Complete |
| TUNE-07 | Phase 3 | Complete |
| MIDI-01 | Phase 4 | Complete |
| MIDI-02 | Phase 4 | Complete |
| MIDI-03 | Phase 4 | Complete |
| MIDI-04 | Phase 4 | Complete |
| AUD-01 | Phase 6 | Pending |
| AUD-02 | Phase 6 | Pending |
| AUD-03 | Phase 6 | Pending |
| AUD-04 | Phase 6 | Pending |
| AUD-05 | Phase 7 | Pending |
| AUD-06 | Phase 6 | Pending |
| VIS-01 | Phase 5 | Pending |
| VIS-02 | Phase 5 | Pending |
| VIS-03 | Phase 5 | Pending |
| VIS-04 | Phase 5 | Pending |
| VIS-05 | Phase 5 | Pending |
| DOC-01 | Phase 8 | Pending |
| DOC-02 | Phase 8 | Pending |
| DOC-03 | Phase 8 | Pending |
| DOC-04 | Phase 8 | Pending |
| DOC-05 | Phase 8 | Pending |
| DOC-06 | Phase 1 | Pending |
| DOC-07 | Phase 3 | Complete |
| DOC-08 | Phase 8 | Pending |
| DIST-01 | Phase 9 | Pending |
| DIST-02 | Phase 9 | Pending |
| DIST-03 | Phase 9 | Pending |
| DIST-04 | Phase 9 | Pending |
| DIST-05 | Phase 9 | Pending |

**Coverage:**
- v1 requirements: 51 total
- Mapped to phases: 51 ✓
- Unmapped: 0

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after initialization*
