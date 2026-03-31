---
plan: 06-02
phase: 06-audio-capture-pitch-detection
status: complete
started: 2026-03-30
completed: 2026-03-30
---

## Summary

Implemented YIN and McLeod NSDF pitch detection algorithms behind the PitchDetector trait, verified by synthetic sine wave tests across the voice/guitar range.

## What Was Built

- **YinDetector**: de Cheveigné & Kawahara 2002 — difference function, CMND, absolute threshold search with local minimum tracking, parabolic interpolation, RMS early bailout
- **McLeodDetector**: McLeod & Wyvill 2005 — NSDF computation via autocorrelation normalization, key maxima selection at zero crossings, clarity threshold, parabolic interpolation
- **Cross-algorithm test suite**: Both detectors agree on 9 test frequencies from E2 (82Hz) to C6 (1047Hz), handle silence, detect fundamentals in harmonic signals

## Key Files

### Created
- `crates/haydn-audio/src/pitch/yin.rs` — YIN pitch detector (130 lines)
- `crates/haydn-audio/src/pitch/mcleod.rs` — McLeod NSDF pitch detector (130 lines)
- `crates/haydn-audio/tests/pitch_accuracy.rs` — Cross-algorithm comparison tests

### Modified
- `crates/haydn-audio/src/pitch/mod.rs` — Added module declarations and re-exports

## Test Results

- 18 unit tests passing (YIN: 5, McLeod: 5, freq_to_midi: 5, types: 3)
- 4 integration tests passing (cross-algorithm agreement, silence, harmonics, confidence)

## Deviations

None.
