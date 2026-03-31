# Phase 9: Distribution & Release - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-31
**Phase:** 09-distribution-release
**Areas discussed:** Version & release identity, Distribution scope, CI & release automation, Esolang wiki publication

---

## Version & Release Identity

| Option | Description | Selected |
|--------|-------------|----------|
| 0.1.0 — first public release | Signals 'real release, try it' — matches Rust ecosystem norms | |
| 1.0.0 — stable v1 | Signals stability and completeness | |
| 0.0.1 — keep current, pre-release | Signals 'proof of concept, expect rough edges' | ✓ |

**User's choice:** 0.0.1 — keep current, pre-release
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — uniform 0.0.1 across all crates | All 5 crates stay at 0.0.1 — consistent and simple | ✓ |
| No — subcrates may version independently later | Main binary at 0.0.1 but subcrates could diverge | |

**User's choice:** Uniform lockstep (after asking for recommendation — agreed that lockstep is simpler for first release, independent versioning can come later if subcrates get external consumers)

---

## Distribution Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Both published, installed separately | `cargo install haydn` + `cargo install haydn-performer` as separate installs | ✓ |
| Only haydn published — performer is internal | Performer stays as dev tool built from source | |
| Both binaries from single crate install | Requires crate restructuring | |

**User's choice:** Both published, installed separately

| Option | Description | Selected |
|--------|-------------|----------|
| crates.io only (cargo install) | Rust users compile from source. Simplest. | ✓ |
| crates.io + pre-built binaries | GitHub Releases with binaries for macOS/Linux/Windows | |
| crates.io + binaries + package managers | Homebrew, AUR, etc. Maximum reach. | |

**User's choice:** crates.io only

---

## CI & Release Automation

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Actions CI — test + build on macOS/Linux/Windows | Full cross-platform CI on every push/PR | ✓ |
| Minimal CI — Linux tests only | Cross-platform validation is manual | |
| No CI — fully manual | Manual local testing and publishing | |

**User's choice:** GitHub Actions CI on all three platforms

| Option | Description | Selected |
|--------|-------------|----------|
| Manual publish — local cargo publish | Simple, no secrets in CI | |
| Automated — tag-triggered publish via CI | GH Action triggered by git tag | |
| Semi-automated — CI creates release, you approve | Draft release model | ✓ |

**User's choice:** Draft release model — after initially expressing interest in automated but raising the concern about tag deletion on test failures. The draft release model was recommended as it avoids the awkward tag-delete flow: tag → test matrix → draft GH Release → manual approve → cargo publish.

---

## Esolang Wiki Publication

| Option | Description | Selected |
|--------|-------------|----------|
| Include wiki submission in Phase 9 | Post after cargo install link is live | ✓ |
| Defer — wiki submission after release | Separate post-release activity | |
| Polish draft only — submit manually later | Draft is the deliverable | |

**User's choice:** Include it — post now, revise later
**Notes:** User initially hesitated about AI-generated content on the wiki. Clarified: esolang wiki is community-contributed with no disclosure requirements; the language/implementation are the user's creative work. User also requested the satirical "working paper / pre-print for peer review" academic framing be preserved on the wiki page (same as README and all docs) — not just the README. The joke is consistent throughout.

---

## Agent's Discretion

- GitHub Actions workflow file structure (single vs separate CI/release)
- CI matrix details (Rust versions, exact OS versions)
- `rust-toolchain.toml` inclusion
- Crate publish delay/retry strategy for index propagation
- `cargo deny` / `cargo audit` in CI
- Release checklist ordering
- `.github/` directory conventions

## Deferred Ideas

None — discussion stayed within phase scope
