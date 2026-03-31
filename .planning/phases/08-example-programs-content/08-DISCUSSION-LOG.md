# Phase 8: Example Programs & Content - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 08-example-programs-content
**Areas discussed:** Program notation, README voice & structure, Example program depth, Esolang wiki page scope

---

## Program Notation

| Option | Description | Selected |
|--------|-------------|----------|
| Event sequence primary | Lead with formal event list (`Push(-24), Op(add)`), note names as annotations | |
| Note names primary | Lead with musical notation (`C2, F2, C4, A5`), event sequences as sidebar | ✓ |
| Multi-format | Ship all representations: events, note names, LilyPond, keyboard diagrams | |
| Tabular walk-through | Step-by-step `Note → Event → Stack → Output` trace for each example | |

**User's choice:** Note names primary, with a one-time tabular walk-through for Hello World only
**Notes:** User initially leaned toward note names. Agent recommended note names primary + tabular trace for Hello World as the teaching moment. User agreed that subsequent examples don't need the full trace once the paradigm is internalized.

---

## README Voice & Structure

### Tone

| Option | Description | Selected |
|--------|-------------|----------|
| Witty-academic | Hook + precise. Humor from absurdity of concept. Conference talk abstract feel. | |
| Theatrical/performative | Second-person present tense. "Sit down at the piano. Play C2..." | |
| Deadpan technical | Matter-of-fact about something absurd. Classic esolang style. | |
| High-energy manifesto | Bold claims, exclamation energy, GIFs, badges. Hype document. | |
| Satirical academic paper | Deadpan formal + absurd content. Research paper structure. | ✓ |

**User's choice:** Satirical academic paper — emerged from user's suggestion of "satirical scientific journal"
**Notes:** User initially liked deadpan technical but found theatrical and witty-academic appealing too. Mentioned "satirical scientific journal" as inspiration. Agent proposed the academic paper framing with Abstract/Introduction/Methods/Results/Discussion/References structure, noting it aligns with the spec's existing voice and Joseph Haydn's wit.

### Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Self-contained | Everything in the README. Long but complete. | |
| Hub with featured content | Hook + Hello World + install in README, everything else linked out. | |
| Layered reveal | Starts minimal, each section deeper. 30s/2min/5min engagement tiers. | ✓ |

**User's choice:** Layered reveal
**Notes:** Agent recommended layered reveal as the best fit for the satirical paper framing — research papers naturally have increasing depth, and the format sustains the comedy without becoming a monolith or feeling hollow.

---

## Example Program Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal | Note sequence + expected output only. Lean, scannable. | |
| Performance guide | Note sequence + tuning file + narrative instructions. | |
| Graduated depth | Hello World full treatment, FizzBuzz moderate, Cat/Truth Machine minimal. | |
| Each teaches a concept | Hello World = push/print, FizzBuzz = loops, Cat = I/O, Truth Machine = conditionals. | ✓ |

**User's choice:** Each example teaches a concept, ordered as a natural progression
**Notes:** User liked the educational framing and mentioned Suzuki method books as inspiration — organizing examples like progressive musical studies with recital pieces. Agent flagged this as scope creep for Phase 8 and suggested deferring "Suzuki-style educational volumes" to a future phase. User agreed.

---

## Esolang Wiki Page Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Comprehensive standalone | Full page understanding Haydn without leaving the wiki. | |
| Standard esolang entry | Overview, instruction table, one example, external links. ~500 words. | |
| Teaser that drives traffic | Short + intriguing, links to repo for everything else. | |
| You decide | Agent's discretion on scope. | |
| Local draft (working paper) | Write comprehensive content in `docs/esolang-wiki.md`, defer publication. | ✓ |

**User's choice:** Local draft as a "working paper" — write the content now, publish in Phase 9
**Notes:** User questioned whether the wiki page should be deferred entirely — wanted to review work before publishing. Agent proposed writing the draft in Phase 8 (while in the content mindset) but deferring publication to Phase 9. User liked the "preprint" framing. Agent corrected to "working paper" since it hasn't been submitted. User appreciated the precision — `<!-- Working paper. Not yet submitted. -->`.

---

## Agent's Discretion

- Exact note sequences for example programs
- README length and exact section content
- Turing completeness proof depth in wiki draft
- Whether to include piano keyboard diagram

## Deferred Ideas

- Suzuki-style educational volumes / graded repertoire (future milestone, pairs with web playground)
- Wiki publication (Phase 9)
- Voice/guitar example programs (future content phase)
