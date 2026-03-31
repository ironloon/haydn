---
phase: 02
slug: vm-core
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `crates/haydn-vm/Cargo.toml` |
| **Quick run command** | `cargo test -p haydn-vm` |
| **Full suite command** | `cargo test -p haydn-vm -- --include-ignored` |
| **Estimated runtime** | ~3 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p haydn-vm`
- **After every plan wave:** Run `cargo test -p haydn-vm -- --include-ignored`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | VM-01, VM-02, VM-03, VM-04, VM-06, VM-07 | unit | `cargo test -p haydn-vm` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | VM-08 | unit | `cargo test -p haydn-vm` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | VM-05 | unit | `cargo test -p haydn-vm` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | VM-01 | integration | `cargo test -p haydn-vm` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test infrastructure exists within `crates/haydn-vm/src/` using `#[cfg(test)]` modules
- [ ] Types module with `Opcode`, `Event`, `StepResult`, `EdgeCase` enums/structs — tests depend on these
- [ ] No external test dependencies needed — Rust's built-in test framework is sufficient

*Wave 0 creates the type definitions that all subsequent tests import.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
