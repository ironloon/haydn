# Phase 9: Distribution & Release - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can install and run Haydn on any major OS with a clean CLI experience. Delivers: crates.io publication for `haydn` and `haydn-performer`, GitHub Actions CI with cross-platform test matrix, tag-triggered release workflow with draft approval, esolang wiki page submission, and repository URL consistency fixes. Does NOT add new CLI features (already complete from Phases 4/5/7), pre-built binaries, or package manager distribution beyond crates.io.

</domain>

<decisions>
## Implementation Decisions

### Version & Release Identity
- **D-01:** **Keep 0.0.1 — pre-release signal.** All 5 crates ship at version 0.0.1. Signals "proof of concept, expect rough edges" which is appropriate for an esolang's first public release.
- **D-02:** **Uniform lockstep versioning across all crates.** `haydn`, `haydn-vm`, `haydn-tuning`, `haydn-audio`, and `haydn-performer` all stay at the same version. No per-crate divergence for v1. If a subcrate (e.g., `haydn-vm`) later gets external consumers, it can diverge at that point.

### Distribution Scope
- **D-03:** **Both `haydn` and `haydn-performer` published to crates.io, installed separately.** Users run `cargo install haydn` for the language, `cargo install haydn-performer` separately for the audio preview tool. Two separate installs, clean separation.
- **D-04:** **crates.io only — no pre-built binaries.** Rust users compile from source. Matches esolang tradition and keeps the release process simple. Pre-built binaries are a future consideration if demand exists.
- **D-05:** **Library crates also published.** `haydn-vm`, `haydn-tuning`, and `haydn-audio` are published as library crates so other projects can depend on them. Path dependencies must be augmented with version pins for crates.io compatibility.

### CI & Release Automation
- **D-06:** **GitHub Actions CI — test + build on macOS, Linux, and Windows.** Every push/PR triggers `cargo test` and `cargo build` across all three platforms. Catches platform-specific issues early.
- **D-07:** **Tag-triggered draft release model.** Workflow: push a git tag (e.g., `v0.0.1`) → CI runs full test matrix → if tests pass, creates a GitHub Release in draft state → user reviews and clicks "Publish release" → that triggers `cargo publish` for all crates in dependency order. Failed tests = no draft = nothing to clean up. No awkward tag-deletion flow.
- **D-08:** **Publish order: `haydn-vm` → `haydn-tuning` → `haydn-audio` → `haydn` → `haydn-performer`.** Respects the dependency graph. Each crate must be live on crates.io before its dependents can publish.
- **D-09:** **`CARGO_REGISTRY_TOKEN` stored as GitHub repo secret.** Moves the token from the local `.env` file to CI. Manual local publishing remains possible as a fallback.

### Esolang Wiki Publication
- **D-10:** **Wiki submission included in Phase 9.** Post the draft from `docs/esolang-wiki.md` to the esolang wiki after `cargo install haydn` link is live. Part of the release checklist — ensures it actually happens.
- **D-11:** **Satirical academic framing preserved everywhere.** Wiki page, README, and all docs maintain the "working paper / pre-print for peer review" voice consistently. The wiki isn't a "real" version — it's the same joke all the way through. The esolang wiki has no tone requirements and this framing is consistent with the project's identity.

### Repository Housekeeping
- **D-12:** **Fix inconsistent repository URLs.** Currently `haydn-vm` and `haydn-tuning` reference `jwgeller/haydn` while `haydn-audio` and `haydn-performer` reference `jaredwray/haydn`. Must be unified to the correct GitHub URL before publishing.
- **D-13:** **Cargo.toml metadata audit.** All crates need complete metadata for crates.io: `description`, `license`, `repository`, `keywords`, `categories`. Most already have this but should be reviewed for consistency.

### Agent's Discretion
- GitHub Actions workflow file structure (single workflow vs separate CI/release workflows)
- CI matrix details (Rust stable/MSRV, exact OS versions)
- Whether to add a `rust-toolchain.toml` for reproducible builds
- Release checklist ordering beyond crate publish (wiki post, GitHub release notes content)
- Whether to add `cargo deny` or `cargo audit` to CI
- Crate publish delay/retry strategy (crates.io index propagation between dependent publishes)
- `.github/` directory structure and naming conventions

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Crate Manifests (publish metadata)
- `Cargo.toml` — Workspace root: member list, resolver config
- `crates/haydn/Cargo.toml` — Main binary crate: dependencies, metadata, path deps needing version pins
- `crates/haydn-vm/Cargo.toml` — VM library: zero external deps, metadata
- `crates/haydn-tuning/Cargo.toml` — Tuning library: depends on haydn-vm, metadata
- `crates/haydn-audio/Cargo.toml` — Audio library: depends on cpal/ringbuf, metadata
- `crates/haydn-performer/Cargo.toml` — Performer binary: heavy deps (rodio, rustysynth, etc.), metadata

### CLI Interface (already complete — verify, don't rebuild)
- `crates/haydn/src/main.rs` — Clap-derived CLI: `--input`, `--tuning`, `--list-midi`, `--list-audio`, `--midi-device`, `--audio-device`, `--quiet`, `--demo`
- `crates/haydn-performer/src/cli.rs` — Performer CLI structure

### Content to Publish
- `docs/esolang-wiki.md` — Draft wiki page ready for submission
- `README.md` — Satirical academic paper style README (Phase 8)
- `docs/SPECIFICATION.md` — Language specification document
- `docs/TUNING.md` — Tuning file format documentation

### Embedded Assets (compiled into binary)
- `crates/haydn-tuning/src/lib.rs` — `include_str!` for piano.toml, voice.toml, guitar.toml
- `crates/haydn-tuning/tunings/piano.toml` — Default piano tuning
- `crates/haydn-tuning/tunings/voice.toml` — Default voice tuning
- `crates/haydn-tuning/tunings/guitar.toml` — Default guitar tuning

### Prior Phase Decisions
- `.planning/phases/04-midi-input-pipeline/04-CONTEXT.md` — D-06/D-07/D-08: device selection CLI pattern (--list-midi, --midi-device)
- `.planning/phases/07-audio-integration-instrument-tunings/07-CONTEXT.md` — D-01/D-03: --input flag, --list-audio, --audio-device flags

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- All CLI flags for DIST-03/DIST-04/DIST-05 already implemented — `--input midi|mic`, `--tuning`, `--list-midi`, `--list-audio`, `--midi-device`, `--audio-device`
- Tuning files embedded via `include_str!` — no runtime file discovery needed for defaults
- Existing `.env` file with `CARGO_REGISTRY_TOKEN` — pattern already established for local publishing

### Established Patterns
- Workspace crate structure with path dependencies — standard for Rust multi-crate projects
- All crates already have `description`, `license`, `repository` fields (though URLs inconsistent)
- `haydn-vm` and `haydn-tuning` are zero-audio-dep libraries — clean publish with no platform-specific build issues
- `haydn-audio` depends on `cpal` which has platform-specific audio backends — CI must validate all three OS targets

### Integration Points
- Path dependencies (`{ path = "../haydn-vm" }`) need `version = "0.0.1"` added for crates.io
- `haydn-performer` is a separate binary crate (`[[bin]]` section) — publishes independently
- Five crates must publish in strict dependency order due to crates.io index requirements

</code_context>

<specifics>
## Specific Ideas

- The satirical academic "working paper / pre-print for peer review" framing should be preserved across all public-facing content (wiki, README, crates.io description) — the joke is consistent and part of the project's identity
- Wiki submission is low-ceremony: post after cargo install link is live, can be completely rewritten/revised anytime later

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-distribution-release*
*Context gathered: 2026-03-31*
