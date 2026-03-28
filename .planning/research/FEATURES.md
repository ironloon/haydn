# Feature Research

**Domain:** Musical esoteric programming language (real-time, performed)
**Researched:** 2026-03-28
**Confidence:** HIGH — based on deep survey of esolang wiki (Brainfuck, Befunge, Piet), live-coding environments (Sonic Pi, TidalCycles, SuperCollider, ChucK), and Wikipedia live coding article

## Feature Landscape

### Table Stakes (Users Expect These)

Features every serious esoteric language must have. Missing any of these and Haydn won't be taken seriously by the esolang community.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Formal language specification** | Every notable esolang (Brainfuck, Befunge, Piet) has a precise, unambiguous spec. This IS the product for the esolang community. | Medium | Must define: instruction set, execution model, I/O behavior, edge cases (division by zero, stack underflow, etc.). Brainfuck's spec ambiguities (EOF handling, cell size) caused decades of incompatible interpreters — learn from this. |
| **Turing completeness proof** | The esolang community will immediately ask "is it Turing complete?" and dismiss languages that can't prove it. Brainfuck, Befunge-98, Piet all have formal proofs. | Medium | Stack-based VM with conditional branching + unbounded storage = provably TC. Provide a reduction from a known TC language or implement a universal Turing machine. |
| **Hello World program** | The first thing anyone tries. Every esolang wiki page shows Hello World. Brainfuck's `Hello World` in minimal form is iconic. Piet's Hello World painting is what makes people share it. | Low | Must be performable — this IS the viral demo. "Watch someone play Hello World on a piano." |
| **Canonical example programs** | Beyond Hello World: Cat program, Truth Machine, FizzBuzz, Fibonacci, 99 Bottles. These are the standard benchmarks from the esolang wiki. | Low-Med | Each program demonstrates different language capabilities. Quine would be especially interesting for a musical language (a melody that programs itself). |
| **Working interpreter** | Must correctly execute all specified operations. Brainfuck has hundreds of interpreters; the reference implementation defines the language. | High | This is the core engineering effort. Real-time audio processing + VM execution in a single low-latency pipeline. |
| **Stack operations** | `push`, `pop`, `dup`, `swap`, `rotate` — standard stack-based VM operations. Befunge and Piet use stacks; Forth-like semantics are well-understood. | Low | Already in PROJECT.md spec. These are the backbone of the computation model. |
| **Arithmetic operations** | `add`, `sub`, `mul`, `div`, `mod` — basic math. Present in every Turing-complete esolang. | Low | Already in spec. Division by zero behavior must be specified explicitly (Befunge asks the user; Brainfuck leaves it undefined — specify a sane default). |
| **Comparison and branching** | `eq`, `gt`, `lt`, conditional jump, unconditional jump — flow control. Without these, Turing completeness is impossible. | Low | Already in spec. The musical mapping of "branching" is a key design decision — intervals? Chord changes? |
| **I/O operations** | Print number, print character, read input. Every esolang needs I/O to produce visible output. | Low | Already in spec. Output goes to terminal; input could be interesting (reading from stdin while also performing?). |
| **Memory operations** | `store`, `load` — random access memory beyond the stack. Needed for non-trivial programs. | Low | Already in spec. Heap/memory addressing scheme must be specified. |
| **Error handling specification** | What happens on stack underflow? Division by zero? Invalid memory access? Brainfuck's undefined behaviors created incompatibility; Befunge specifies most edge cases. | Low | Specify ALL edge cases in the language spec. Musical esolang adds unique edge cases: what if pitch detection fails? What if no note is detected? |
| **CLI binary distribution** | `cargo install haydn` — single command to get started. Low friction is essential for adoption. | Low | Already planned. Rust's cargo ecosystem makes this straightforward. |
| **README in esolang tradition** | Fun, irreverent, informative. Look at Brainfuck's history, Piet's artistic angle, Shakespeare's dramatic flair. The README is the recruiting tool. | Low | Should open with the concept ("What if music was code?"), show a compelling example, and link to the spec. Humor matters. |

### Differentiators (What Makes Haydn Special)

Features that set Haydn apart from both traditional esolangs and from live-coding tools. These create the "holy shit" moment.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Musical input AS program source** | The core innovation. No esolang uses real-time musical performance as input. Piet uses images, Befunge uses 2D text, Chef uses recipes — Haydn uses live music. This is the "thematic gimmick" that makes it memorable. | High | This is not just a feature — it IS the language. The entire architecture exists to serve this. |
| **Configurable tuning files** | Different instruments/styles map differently to opcodes. A pianist's mapping differs from a vocalist's. This is like Brainfuck's "trivial substitution" concept but much deeper — the same computation can be performed as different music. | Medium | TOML/YAML config files mapping note ranges, intervals, and gestures to VM opcodes. Ship with defaults for keyboard, voice, and guitar. |
| **Dual input: MIDI + microphone** | MIDI is clean/discrete (easy to map, reliable); microphone is expressive/accessible (anyone with a voice can program). No other system bridges both worlds in a single language. | High | MIDI is straightforward event handling. Microphone requires real-time pitch detection (YIN algorithm), onset detection, and noise handling. Two very different engineering challenges. |
| **Real-time pitch detection** | Converting continuous audio (singing, playing) into discrete note events that map to opcodes. This is what makes the microphone input magical. | High | YIN or pYIN algorithm for monophonic pitch detection. Must handle: vibrato, pitch drift, ambient noise, note onset/offset. Latency target: <20ms for perceived immediacy. |
| **CLI visualization of VM state** | Watch the stack grow as you play. See operations execute. See output appear. The visual feedback loop is what makes the performance compelling as a live coding tool. | Medium | Render: current stack contents, last operation executed, output buffer, active notes. TUI library (ratatui or similar) for real-time terminal rendering. |
| **The performance IS the program** | In Sonic Pi / TidalCycles, you type code that generates music. In Haydn, you PLAY music that generates computation. The directionality is reversed. This is philosophically novel. | N/A (design) | This is a design principle, not a feature. But it drives every implementation decision. |
| **Musical Turing completeness demos** | Demonstrate that a melody IS a computation. Video of someone playing FizzBuzz on piano. Audio recording of someone singing Hello World. These are the shareable artifacts that make Haydn go viral. | Low | These are content, not code. But they're the most important "feature" for adoption. Plan them as first-class deliverables. |
| **Esolang wiki page** | A well-crafted esolang wiki page is how the esolang community discovers new languages. Brainfuck's page is the most-visited on the wiki. | Low | Include: language spec summary, instruction table, computational class proof, example programs, links to interpreter. |
| **Named after Haydn** | Joseph Haydn was the "father of the symphony" and known for wit and surprise (Surprise Symphony, Farewell Symphony). The name creates immediate cultural association and suggests the language is both serious and playful. | N/A (design) | Lean into this in docs. Quote Haydn. Reference his innovations. "The first truly musical programming language, named after the first great symphonist." |
| **Instrument-specific default tunings** | Ship with pre-made tuning files for common instruments. A pianist can start immediately; a guitarist can start immediately. Lower the barrier to the first "Hello World" performance. | Low | Piano: octave-based mapping (C4=push 0, C#4=push 1, intervals=operations). Voice: pitch range mapping. Guitar: string/fret mapping. |
| **Note-as-value semantic** | Notes push their MIDI number (or mapped value) onto the stack. Musical intervals between consecutive notes trigger operations. This duality (notes=data, intervals=operations) is elegant and uniquely musical. | Medium | Core language design decision. Must be specified precisely. Edge cases: what does a rest (silence) mean? What about sustained notes? |

### Anti-Features (Deliberately NOT Building)

Features that seem logical but would dilute the vision, bloat v1, or emerge from the wrong mental model.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Audio output / sonification** | "If input is music, output should be music too!" | Reverses the fundamental concept. Haydn's magic is that music becomes computation, not that computation becomes music. Sonic Pi already does music output brilliantly. Adding it here makes Haydn a worse Sonic Pi instead of a unique esolang. | Output is text/visual via the CLI. The PERFORMANCE is the music. The RESULT is the computation. |
| **Web playground / WASM** | "I want to try it without installing anything" | Major engineering effort (WebAudio API, WASM compilation, web MIDI). Splits focus from the core real-time Rust implementation. | Ship a solid `cargo install` experience and compelling demo videos. Web playground is a future milestone. |
| **GUI / rich TUI** | "Make the visualization pretty" | GUI frameworks add massive dependency weight and platform complexity. The CLI aesthetic matches the esolang tradition (Brainfuck doesn't need a GUI). | Simple but effective CLI visualization. Think `htop` not Ableton. Ratatui for terminal rendering is sufficient. |
| **Multi-performer / ensemble mode** | "What if multiple musicians program together?" | Network synchronization, state merging, concurrent VM execution — enormous complexity. SuperCollider's Republic system took years to develop. | Single performer for v1. The solo performance is compelling on its own. Ensemble is a v2+ feature. |
| **Recording / playback** | "I want to save and replay my performance" | A recording isn't a program in the traditional sense — it's a sequence of timed events. Adding a serialization format is scope creep. | Users can record MIDI externally (any DAW). The language spec + tuning file IS the "source code." |
| **General-purpose language features** | "Add string operations, file I/O, networking" | This is an esoteric language. Brainfuck has 8 commands. Piet has 17 commands. General-purpose features destroy the constraint that makes esolangs interesting. | Keep the instruction set minimal and elegant. If someone needs general-purpose, they should use a general-purpose language. |
| **IDE / editor support** | "Add syntax highlighting, autocomplete" | There is no text to edit. The "source code" is a musical performance. IDE support is the wrong paradigm. | The CLI tool IS the IDE. You start it, you play, it executes. |
| **Package manager / module system** | "Let people share and import tuning files" | Over-engineering for v1. Tuning files are simple config files that can be shared as files. | Ship defaults. Document the tuning file format. Let people share via GitHub/gist. |
| **Chord recognition** | "What about playing chords as compound operations?" | Polyphonic pitch detection from microphone is unsolved in real-time. MIDI can detect chords trivially, but asymmetry between input sources is confusing. | Keep it monophonic for v1 (single note at a time). Intervals between consecutive notes drive operations. |
| **Tempo / rhythm awareness** | "Use rhythm and timing as part of the language semantics" | Massively increases complexity. Rhythm detection, beat tracking, tempo estimation — each is a research problem. The language should respond to WHAT you play, not HOW FAST. | Notes trigger operations immediately on detection. Timing is irrelevant to semantics (you can play fast or slow). |

## Feature Dependencies

```
Language Specification
├── Instruction Set Definition
│   ├── Stack Operations (fundamental)
│   ├── Arithmetic Operations (depends on stack)
│   ├── Comparison/Branching (depends on stack + arithmetic)
│   ├── I/O Operations (depends on stack)
│   └── Memory Operations (depends on stack)
├── Tuning File Format (maps musical events → instructions)
│   ├── Default Piano Tuning (depends on tuning format)
│   ├── Default Voice Tuning (depends on tuning format)
│   └── Default Guitar Tuning (depends on tuning format)
└── Edge Case Specification (depends on all operations)

VM Implementation
├── Stack Engine (core, no dependencies)
├── Instruction Executor (depends on stack engine)
├── Memory Subsystem (depends on instruction executor)
└── I/O Subsystem (depends on instruction executor)

Input Pipeline
├── MIDI Input Handler (independent)
│   └── MIDI → Instruction Mapper (depends on tuning files + VM)
├── Microphone Input Handler (independent)
│   ├── Pitch Detection Engine (depends on mic input)
│   ├── Onset Detection (depends on mic input)
│   └── Audio → Instruction Mapper (depends on pitch detection + tuning files + VM)
└── Input Source Selector (depends on both handlers)

CLI Visualization
├── Stack Display (depends on VM)
├── Operation Log (depends on VM)
├── Output Display (depends on VM I/O)
└── Terminal Renderer (depends on all display components)

Documentation & Content
├── Language Spec Document (depends on language specification)
├── README (depends on working interpreter)
├── Example Programs (depends on working interpreter + tuning files)
│   ├── Hello World (first priority)
│   ├── FizzBuzz (second priority)
│   ├── Cat Program (second priority)
│   └── Truth Machine (second priority)
├── Esolang Wiki Page (depends on spec + examples)
└── Turing Completeness Proof (depends on spec + VM)
```

## MVP Definition

### Launch With (v1.0)

The minimum viable esolang that demonstrates the concept and lets performers program.

1. **Language specification** — Complete, unambiguous, covering all instructions and edge cases
2. **Stack-based VM** — All 20+ opcodes working correctly
3. **MIDI input** — Clean, reliable, low-latency note-to-instruction mapping
4. **Default piano tuning file** — At least one tuning file that works out of the box
5. **Tuning file format** — Documented, so users can create their own
6. **CLI visualization** — Stack state, current operation, output display
7. **Hello World example** — The "first program" that demonstrates the concept
8. **Turing completeness proof** — Mathematical credibility
9. **README** — Fun, clear, compelling — the esolang community recruiting tool
10. **`cargo install haydn`** — Frictionless distribution

### Add After Validation (v1.x)

Features that enhance the experience once the core concept is proven.

1. **Microphone input with pitch detection** — The harder but more accessible input path (could ship with MIDI-only v1 and add mic in v1.1)
2. **Default voice tuning file** — Once mic input works
3. **Default guitar tuning file** — Once mic input works
4. **Additional example programs** — FizzBuzz, Cat, Truth Machine, Fibonacci, 99 Bottles
5. **Esolang wiki page** — After v1 is stable and documented
6. **Performance demo videos** — After real performers have used it
7. **Improved CLI visualization** — Richer display based on user feedback

### Future Consideration (v2+)

1. **Web playground (WASM)** — Browser-based interpreter with Web MIDI / WebAudio
2. **Multi-performer mode** — Networked ensemble programming
3. **Recording / playback** — Serialize performances for replay
4. **Chord recognition (MIDI)** — Compound operations from simultaneous notes
5. **Self-interpreter** — A Haydn program that interprets Haydn (the ultimate flex, like Brainfuck's self-interpreters)
6. **Additional tuning files** — Drums, wind instruments, exotic instruments
7. **Rich TUI/GUI visualization** — Beyond basic CLI

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Language specification | Essential (no spec = no language) | Medium (design work) | **P0 - Must** |
| Stack-based VM | Essential (the engine) | High (core engineering) | **P0 - Must** |
| MIDI input handler | Essential (primary input for v1) | Medium (well-understood protocol) | **P0 - Must** |
| Tuning file format + parser | Essential (configurable mapping) | Low-Medium | **P0 - Must** |
| Default piano tuning | Essential (out-of-box experience) | Low | **P0 - Must** |
| CLI visualization | High (the feedback loop) | Medium | **P0 - Must** |
| Hello World example | High (first demo) | Low | **P0 - Must** |
| Turing completeness proof | High (credibility) | Medium | **P0 - Must** |
| README | High (discovery/adoption) | Low | **P0 - Must** |
| cargo install distribution | High (frictionless setup) | Low | **P0 - Must** |
| Microphone input + pitch detection | High (accessibility) | High (audio DSP) | **P1 - Should** |
| Default voice tuning | Medium | Low (once mic works) | **P1 - Should** |
| Default guitar tuning | Medium | Low (once mic works) | **P1 - Should** |
| Additional example programs | Medium (depth) | Low each | **P1 - Should** |
| Esolang wiki page | Medium (community visibility) | Low | **P1 - Should** |
| Edge case specification | Medium (interoperability) | Low | **P1 - Should** |
| Performance demo videos | High (virality) | Low (content, not code) | **P1 - Should** |
| Web playground | Medium (accessibility) | Very High | **P2 - Later** |
| Multi-performer mode | Low (niche) | Very High | **P2 - Later** |
| Recording/playback | Low | Medium | **P2 - Later** |
| Self-interpreter | Low (bragging rights) | High | **P2 - Later** |

## Key Insight: The Two Audiences

Haydn sits at the intersection of two communities with different expectations:

**Esolang enthusiasts** care about:
- Precise specification
- Turing completeness
- Clever design constraints
- Example programs (Hello World, quines, self-interpreters)
- Humor and thematic coherence
- Esolang wiki presence

**Music technologists / creative coders** care about:
- Low-latency real-time response
- MIDI integration
- Accessible setup
- Visual feedback during performance
- Performability (can you actually use this live?)
- Community and sharing

**The overlap** — and Haydn's sweet spot — is: "A computationally serious language that is genuinely fun to perform." Both audiences want to be surprised and delighted. Both want to share cool things they found. The Hello World played on piano is the artifact that bridges both worlds.

## Sources

- Esolang Wiki: Brainfuck — https://esolangs.org/wiki/Brainfuck (conventions, implementations, example programs, spec issues)
- Esolang Wiki: Befunge — https://esolangs.org/wiki/Befunge (2D esolang patterns, stack-based VM, visualizers)
- Esolang Wiki: Piet — https://esolangs.org/wiki/Piet (non-textual esolang, visual encoding, stack-based, Turing completeness proof via BF interpreter)
- Esolang Wiki: Main Page — https://esolangs.org/wiki/Main_Page (7,032 registered esolangs, community norms)
- Wikipedia: Live Coding — https://en.wikipedia.org/wiki/Live_coding (environment survey, techniques, TOPLAP manifesto)
- Sonic Pi — https://sonic-pi.net/ (live coding features: MIDI I/O, OSC, Ableton Link, synths, effects, live loops)
- TidalCycles — https://tidalcycles.org/ (pattern-based live coding, SuperCollider backend, algorithmic patterns)
- PROJECT.md — Local project context file defining requirements and constraints
