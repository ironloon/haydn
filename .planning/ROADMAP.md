# Roadmap: Haydn

## Overview

Haydn is built from the spec outward: a formal language specification establishes the semantics, a VM implements them, and then layers of input (MIDI first for clean discrete events, then microphone for expressive continuous signal) progressively bring the language to life. Visualization makes the machine observable mid-performance. The final phases create compelling demo content and ship the binary. Every phase delivers a testable increment — a working MIDI demo exists by Phase 4, and the full "sing into your laptop and compute FizzBuzz" experience arrives by Phase 8.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Language Specification** - Define the complete, unambiguous formal spec including edge cases, live-input semantics, and Turing completeness proof
- [ ] **Phase 2: VM Core** - Implement the stack-based virtual machine with all 20+ opcodes and graceful edge case handling
- [ ] **Phase 3: Tuning System** - Build the configurable tuning file format, parser, validator, default piano tuning, and tuning documentation
- [ ] **Phase 4: MIDI Input Pipeline** - Capture real-time MIDI events and wire them through the tuning engine into the VM for first playable experience
- [ ] **Phase 5: CLI Visualization** - Real-time terminal UI displaying stack state, operations, and output during live performance
- [ ] **Phase 6: Audio Capture & Pitch Detection** - Microphone input with pitch detection, onset detection, and noise gating to produce clean note events
- [ ] **Phase 7: Audio Integration & Instrument Tunings** - Wire microphone events through the full pipeline and create voice and guitar default tunings
- [ ] **Phase 8: Example Programs & Content** - Hello World, FizzBuzz, Cat, Truth Machine examples plus README and esolang wiki page
- [ ] **Phase 9: Distribution & Release** - cargo install, cross-platform builds, CLI interface with device/tuning selection

## Phase Details

### Phase 1: Language Specification
**Goal**: Complete, unambiguous formal specification of the Haydn language that leaves no room for implementation guesswork
**Depends on**: Nothing (first phase)
**Requirements**: SPEC-01, SPEC-02, SPEC-03, SPEC-04, SPEC-05, SPEC-06, SPEC-07, SPEC-08, DOC-06
**Success Criteria** (what must be TRUE):
  1. A developer can read the language specification and know exactly what every instruction does, including all edge cases (stack underflow, division by zero, invalid memory)
  2. Jump/loop semantics for live-input context are formally defined — how branching works when the program is generated in real time
  3. A Turing completeness proof exists via reduction from a known TC language (e.g., Brainfuck)
  4. Note-as-value and interval-as-operation mappings are specified as formal rules with no ambiguity
**Plans**: 2 plans
Plans:
- [x] 01-01-PLAN.md — VM core spec: computational model, instruction set, edge cases, event interface, loop state machine
- [x] 01-02-PLAN.md — Turing completeness proof, examples, and published specification document

### Phase 2: VM Core
**Goal**: Working stack-based virtual machine that correctly executes all Haydn instructions
**Depends on**: Phase 1
**Requirements**: VM-01, VM-02, VM-03, VM-04, VM-05, VM-06, VM-07, VM-08
**Success Criteria** (what must be TRUE):
  1. All opcodes (push, dup, swap, drop, rotate, add, sub, mul, div, mod, eq, gt, lt, jump, conditional jump, loop, print number, print char, read, store, load) execute correctly when fed a stream of instructions
  2. Edge cases (stack underflow, division by zero, invalid memory access, empty stack) are handled gracefully per the language spec — no panics
  3. Control flow (jumps, conditionals, loops) works correctly under the live-input execution model defined in Phase 1
  4. VM can be driven by synthetic instruction sequences in unit tests, validating every opcode independently
**Plans**: 2 plans
Plans:
- [x] 02-01-PLAN.md — Types + VM struct + all 17 non-loop opcodes with tests
- [ ] 02-02-PLAN.md — Loop pedal state machine + integration tests + BF validation

### Phase 3: Tuning System
**Goal**: Configurable tuning files that map musical events to VM instructions, with a working piano default
**Depends on**: Phase 2
**Requirements**: TUNE-01, TUNE-02, TUNE-03, TUNE-06, TUNE-07, DOC-07
**Success Criteria** (what must be TRUE):
  1. User can write a TOML tuning file that maps notes and intervals to any VM opcode
  2. Default piano tuning ships with the binary and maps keyboard notes/intervals to the full instruction set
  3. Invalid tuning files produce clear error messages explaining exactly what is misconfigured
  4. Tuning file documentation with examples enables users to create custom instrument mappings
**Plans**: 3 plans
Plans:
- [x] 03-01-PLAN.md — Crate scaffold, TOML parsing, validation pipeline
- [x] 03-02-PLAN.md — Rule engine, default piano tuning, integration tests
- [x] 03-03-PLAN.md — Tuning documentation (format reference, tutorial, examples)

### Phase 03.1: Simulated Performer (INSERTED)

**Goal:** Standalone audio preview tool that plays LilyPond notation through layered synthesis backends (built-in, SoundFont, MIDI), enabling musicality evaluation of tuning note assignments before real hardware exists
**Depends on:** Phase 3
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-08, D-09, D-10, D-11, D-12, D-13, D-14, D-15, D-16, D-17, D-18, D-19, D-20
**Success Criteria** (what must be TRUE):
  1. User can run `cargo run -p haydn-performer -- score.ly` and hear audio through computer speakers
  2. LilyPond notation is parsed to MIDI note events with correct pitch and duration
  3. Built-in synthesis produces musical audio with ADSR envelope shaping, waveform variety, and expressive features
  4. SoundFont mode renders realistic instrument sounds from .sf2 files
  5. MIDI output mode writes .mid files or sends to virtual MIDI ports
  6. TUI displays current note, progress bar, and measure tracking during playback
**Plans**: 4 plans
Plans:
- [x] 03.1-01-PLAN.md — Crate scaffold + LilyPond subset parser with nom
- [x] 03.1-02-PLAN.md — Built-in synthesis (L0-L1) + CLI + audio playback
- [x] 03.1-03-PLAN.md — Rich synthesis (L2-L3) + SoundFont backend (L4)
- [x] 03.1-04-PLAN.md — MIDI output (L5) + TUI display + example scores

### Phase 4: MIDI Input Pipeline
**Goal**: First end-to-end playable experience — MIDI keyboard notes become live computation
**Depends on**: Phase 3
**Requirements**: MIDI-01, MIDI-02, MIDI-03, MIDI-04
**Success Criteria** (what must be TRUE):
  1. User can connect any MIDI device and see note-on/note-off events captured in real time
  2. MIDI events flow through the active tuning file and execute VM instructions end-to-end
  3. Input-to-execution latency is imperceptible during live performance (<5ms)
  4. Disconnecting or reconnecting a MIDI device is handled gracefully without crashing
**Plans**: 2 plans
Plans:
- [x] 04-01-PLAN.md — MIDI capture + full tuning→VM→event log pipeline (happy path)
- [x] 04-02-PLAN.md — Disconnect/reconnect resilience + Ctrl+C shutdown + human verification

### Phase 5: CLI Visualization
**Goal**: Real-time terminal display of VM state so performers can see what their music is computing
**Depends on**: Phase 4
**Requirements**: VIS-01, VIS-02, VIS-03, VIS-04, VIS-05
**Success Criteria** (what must be TRUE):
  1. User can see stack contents update in real time as they play MIDI notes
  2. The last executed operation and program output (printed numbers/characters) are visible in the TUI
  3. Visualization updates immediately on each musical event with no perceptible lag
  4. TUI rendering does not degrade audio processing or input capture performance
**Plans**: 3 plans
Plans:
- [x] 05-01-PLAN.md — TUI display module: state model, three-panel rendering, loop_state accessor
- [x] 05-02-PLAN.md — Main loop integration: TUI/quiet dual mode, disconnect handling in TUI
- [x] 05-03-PLAN.md — Human verification: live MIDI performance with TUI dashboard

### Phase 6: Audio Capture & Pitch Detection
**Goal**: Microphone input produces clean, discrete note events from continuous audio signal
**Depends on**: Phase 2
**Requirements**: AUD-01, AUD-02, AUD-03, AUD-04, AUD-06
**Success Criteria** (what must be TRUE):
  1. User can sing or play into a microphone and see detected note events in real time
  2. Pitch detection correctly identifies monophonic notes from the audio signal
  3. Onset detection distinguishes new notes from sustained sound — held notes don't spam events
  4. Ambient noise does not trigger spurious note events (noise gate works)
  5. End-to-end audio capture pipeline latency is acceptable for live performance (<60ms)
**Plans**: 4 plans
Plans:
- [x] 06-01-PLAN.md — Crate scaffold + types + PitchDetector trait + tuning [audio] section
- [x] 06-02-PLAN.md — YIN + McLeod NSDF pitch detection implementations + accuracy tests
- [x] 06-03-PLAN.md — Onset detection + noise gate + note state machine
- [x] 06-04-PLAN.md — Audio capture pipeline + public API + integration tests + verification

### Phase 7: Audio Integration & Instrument Tunings
**Goal**: Microphone performance drives the full Haydn pipeline with voice and guitar tunings
**Depends on**: Phase 5, Phase 6
**Requirements**: AUD-05, TUNE-04, TUNE-05
**Success Criteria** (what must be TRUE):
  1. User can perform via microphone and see real-time computation through the full pipeline (audio → tuning → VM → TUI)
  2. Default voice tuning works for singing into a microphone — vocal ranges map to a usable instruction set
  3. Default guitar tuning works for acoustic guitar played into a microphone
**Plans**: 3 plans
Plans:
- [x] 07-01-PLAN.md — Instrument tuning files (voice.toml, guitar.toml) + loaders + AudioSection accessor
- [x] 07-02-PLAN.md — Audio types + Cargo dep + AudioConfig helper + TUI display model
- [x] 07-03-PLAN.md — CLI flags + main loop dual-input + signal level + disconnect/reconnect

### Phase 8: Example Programs & Content
**Goal**: Compelling demo programs and documentation that showcase Haydn to the esolang and music-tech communities
**Depends on**: Phase 7
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, DOC-08
**Success Criteria** (what must be TRUE):
  1. Hello World can be performed on a piano/keyboard using the default tuning, with clear instructions
  2. FizzBuzz, Cat program, and Truth Machine are performable examples with tuning files and instructions
  3. README is fun, irreverent, and technically precise — in the tradition of great esolangs
  4. Esolang wiki page exists with spec summary, instruction table, examples, and interpreter link
**Plans**: TBD

### Phase 9: Distribution & Release
**Goal**: Users can install and run Haydn on any major OS with a clean CLI experience
**Depends on**: Phase 8
**Requirements**: DIST-01, DIST-02, DIST-03, DIST-04, DIST-05
**Success Criteria** (what must be TRUE):
  1. User can install Haydn via `cargo install haydn` and it just works
  2. Binary runs correctly on macOS, Linux, and Windows
  3. `haydn --input midi` and `haydn --input mic` start a performance session with the correct input source
  4. User can select a tuning file via `--tuning` flag and list available MIDI devices and audio inputs
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9
Note: Phase 6 can begin after Phase 2 (parallel with Phases 3-5 if desired).

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Language Specification | 0/0 | Not started | - |
| 2. VM Core | 0/0 | Not started | - |
| 3. Tuning System | 0/0 | Not started | - |
| 4. MIDI Input Pipeline | 0/0 | Not started | - |
| 5. CLI Visualization | 0/0 | Not started | - |
| 6. Audio Capture & Pitch Detection | 0/0 | Not started | - |
| 7. Audio Integration & Instrument Tunings | 0/0 | Not started | - |
| 8. Example Programs & Content | 0/0 | Not started | - |
| 9. Distribution & Release | 0/0 | Not started | - |

## Backlog

### Phase 999.1: Human Voice Synthesis (BACKLOG)

**Goal:** Real-time human voice simulation engine — formant synthesis, vowel/consonant modeling, and expressive vocal parameters (vibrato, breathiness, chest/head register). Initially standalone, then integrated as a performer synth backend. Could also serve as a general-purpose vocal synthesis library for other projects.

**Context & Notes:**
- Formant synthesis (parallel/cascade resonator banks) is the classic approach — models the vocal tract as a set of tunable bandpass filters
- Key formants: F1 (jaw openness, 200-800Hz), F2 (tongue position, 600-2500Hz), F3 (lip rounding, 1800-3500Hz)
- Vowel space: each vowel maps to specific F1/F2/F3 frequencies (e.g., /a/ ≈ 730/1090/2440, /i/ ≈ 270/2290/3010)
- Consonants need noise generators (fricatives), transient bursts (plosives), nasal resonators
- Expressive parameters: vibrato rate/depth, breathiness (noise mix), register (chest vs head), glottal pulse shape
- Modern alternative: neural TTS (Tacotron/VITS) — much more realistic but heavyweight, GPU-dependent
- Middle ground: rustysynth choir patches (GM 52-54) provide basic vocal sounds via SoundFont now
- For the Haydn performer specifically: voice synth would let users hear what their sung programs "should" sound like
- Potential standalone crate: `haydn-voice` or `formant-synth` — useful beyond this project

**Research Areas:**
- Klatt synthesizer algorithm (classic formant synthesis, well-documented)
- WORLD vocoder (Rust bindings exist?) — analysis/synthesis of voice parameters
- Fox/Praat formant extraction for reference data
- Real-time constraints: can formant synthesis run at audio rate (44.1kHz) without glitches?

**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd-review-backlog when ready)

---
*Roadmap created: 2026-03-28*
*Last updated: 2026-03-29*
