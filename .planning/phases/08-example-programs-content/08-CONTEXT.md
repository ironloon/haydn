# Phase 8: Example Programs & Content - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Compelling demo programs and documentation that showcase Haydn to the esolang and music-tech communities. Delivers: Hello World, FizzBuzz, Cat, and Truth Machine example programs; a README in satirical academic paper style; and a draft esolang wiki page. Does NOT publish the wiki page (deferred to Phase 9) or create a pedagogical curriculum.

</domain>

<decisions>
## Implementation Decisions

### Program Notation
- **D-01:** Note names are the primary representation in all documentation. Lead with musical notation (`C2, F2, C4, A5`), not event sequences (`Push(-24), Op(add)`). The performance IS the program — notation should reinforce that.
- **D-02:** Event sequences (`Push(v)`, `Op(opcode)`) appear as inline annotations where helpful, not as the lead format.
- **D-03:** Hello World gets a one-time tabular walk-through (`Note → Event → Stack State → Output`) to teach the paradigm. Subsequent examples do not repeat this — readers have internalized the mapping.
- **D-04:** Existing `.ly` files in `haydn-performer/examples/` provide LilyPond notation for anyone who wants musical score format. No new notation formats needed.

### README Voice & Structure
- **D-05:** Satirical academic paper framing — deadpan formal tone, absurd content. Humor comes from treating musical programming with the gravity of a research paper. Consistent with the spec's existing voice.
- **D-06:** Section headers mirror a research paper: Abstract, Introduction, Methods (how it works), Results (examples), Discussion (tuning system, future work), References (spec, esolang wiki).
- **D-07:** Layered reveal structure — depth increases as reader scrolls. Abstract is 30 seconds; Introduction + Hello World is 2 minutes; the full "paper" is 5 minutes. Detailed content links out to `docs/` and examples.
- **D-08:** Joseph Haydn references are welcome (Surprise Symphony, Farewell Symphony) but not forced. The wit should be structural, not cutesy.

### Example Program Depth
- **D-09:** Each example teaches a concept, ordered as a natural progression: Hello World (push/operate/print) → Cat (I/O) → Truth Machine (conditional loops) → FizzBuzz (loops + modular arithmetic).
- **D-10:** Graduated detail is inherent — Hello World gets the full tabular trace treatment; later examples build on established understanding and need less scaffolding.
- **D-11:** All examples use the default piano tuning and note-name notation. Voice/guitar examples are out of scope for Phase 8 (the tunings exist, but example programs target piano first).

### Esolang Wiki Page
- **D-12:** Wiki page written as a local draft at `docs/esolang-wiki.md` — a "working paper, not yet submitted" in the satirical academic framing.
- **D-13:** Comprehensive enough to satisfy DOC-08 (spec summary, instruction table, Hello World example, Turing completeness note, interpreter link). Actual wiki publication deferred to Phase 9.

### Agent's Discretion
- Exact note sequences for each example program (researcher/planner will work these out against the piano tuning)
- README length and exact section content within the satirical paper structure
- How much of the Turing completeness proof to include in the wiki draft vs. linking to the spec
- Whether to include a piano keyboard diagram in the README

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Language & VM
- `docs/SPECIFICATION.md` — Complete VM specification: all 19 opcodes, event interface, loop pedal state machine, edge cases, Turing completeness proof
- `docs/SPECIFICATION.md` §3 — Instruction set and opcode table (needed for example program construction)
- `docs/SPECIFICATION.md` §5 — Loop pedal model (needed for FizzBuzz and Truth Machine examples)
- `docs/SPECIFICATION.md` §7 — Turing completeness proof via Brainfuck reduction

### Tuning System
- `crates/haydn-tuning/tunings/piano.toml` — Default piano tuning: value zone (MIDI 36-59), operation zone (MIDI 60+, white keys = C major opcodes)
- `crates/haydn-tuning/tunings/voice.toml` — Default voice tuning (reference only, not used in Phase 8 examples)
- `crates/haydn-tuning/tunings/guitar.toml` — Default guitar tuning (reference only)
- `docs/TUNING.md` — Tuning file format documentation

### Existing Examples
- `crates/haydn-performer/examples/hello-world.ly` — Existing LilyPond sketch of Hello World on piano tuning
- `crates/haydn-performer/examples/fizzbuzz-intro.ly` — Existing LilyPond sketch of FizzBuzz opening measures

### Project Identity
- `.planning/PROJECT.md` — Core value: "The live performance IS the program"
- `.planning/REQUIREMENTS.md` — DOC-01 through DOC-08 requirements for this phase

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `haydn-performer/examples/*.ly`: 11 LilyPond scores already exist, including `hello-world.ly` and `fizzbuzz-intro.ly` which directly correspond to Phase 8 example programs
- `haydn-vm` crate: The VM can be driven by synthetic event sequences in tests — example programs can be verified by feeding events and checking output
- Piano tuning (`piano.toml`): Root = C4 (MIDI 60), values = MIDI 36-59 (push semitone offsets -24 to -1), operations = white keys in C major for 19 opcodes

### Established Patterns
- The spec already uses the satirical academic voice (opens with "Abstract. Haydn is an esoteric programming language...")
- Prior phase context files establish musicality-first design: common instruction sequences should sound pleasant in C major
- Source independence guarantee: MIDI and audio produce identical events, so examples work with any input

### Integration Points
- `README.md` at repo root (currently placeholder: "# haydn / in the loft")
- `docs/` directory for the wiki draft alongside existing `SPECIFICATION.md` and `TUNING.md`
- `examples/` directory at repo root for standalone example program files (separate from `haydn-performer/examples/` which are LilyPond scores)

</code_context>

<specifics>
## Specific Ideas

- README Abstract should read like a real research paper abstract — deadpan description of something absurd. Reference the mock abstract discussed: "We present Haydn, a novel programming language in which programs are performed rather than written..."
- Hello World is THE flagship demo — "Hello World emerging letter by letter" in the TUI is the target visual (from Phase 5 context)
- "Music is Art" philosophy: tuning file + performance description combos are shareable "seeds" for exploratory discovery — the README should convey this
- Imprecision-as-feature for voice: "Your voice is unique, so is your program" — mention in README Discussion section
- Wiki draft header: `<!-- Working paper. Not yet submitted. -->` — honest and in character

</specifics>

<deferred>
## Deferred Ideas

- **Suzuki-style educational volumes / graded repertoire** — Organizing examples into progressive difficulty books with suggested recital pieces. Pairs well with a v2 web playground where learners could work through lessons interactively. Future milestone.
- **Wiki publication** — Actual submission to the Esolang Wiki deferred to Phase 9 (Distribution & Release).
- **Voice/guitar example programs** — Piano examples come first; voice and guitar performance guides could be a future content phase.

</deferred>

---

*Phase: 08-example-programs-content*
*Context gathered: 2026-03-30*
