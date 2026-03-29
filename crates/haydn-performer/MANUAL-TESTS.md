# Manual Test Checklist — haydn-performer

These tests verify things that automated tests **cannot**: actual audible output,
TUI rendering, and subjective audio quality. Run these after any synthesis changes.

## Audio Output Tests

### 1. Basic Audio Diagnostic
```
cargo run -p haydn-performer -- --test-audio
```
- [ ] You hear 4 distinct triads (C-E-G played 4 times)
- [ ] Each successive fidelity level sounds different:
  - Fidelity 0: clean pure sine (electronic beep)
  - Fidelity 1: shaped sine (notes fade in/out naturally)
  - Fidelity 2: richer tone (more "instrument-like")
  - Fidelity 3: slight wobble/vibrato audible

### 2. Scale Playback
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/scale.ly --quiet
```
- [ ] You hear ascending then descending C major scale
- [ ] Notes are distinct (not slurred together)
- [ ] No clicks or pops between notes

### 3. Fidelity Comparison
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/hello-world.ly --quiet --fidelity 0
cargo run -p haydn-performer -- crates/haydn-performer/examples/hello-world.ly --quiet --fidelity 3
```
- [ ] Fidelity 0 sounds noticeably more "plain" than fidelity 3
- [ ] Fidelity 3 has audible vibrato/richness

### 4. Tempo Control
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/scale.ly --quiet --bpm 60
cargo run -p haydn-performer -- crates/haydn-performer/examples/scale.ly --quiet --bpm 240
```
- [ ] BPM 60 plays noticeably slower (~16 seconds for scale)
- [ ] BPM 240 plays noticeably faster (~4 seconds for scale)

## TUI Display Tests

### 5. TUI Playback
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/hello-world.ly --bpm 80
```
- [ ] TUI appears with bordered box
- [ ] Current note is highlighted with brackets: `[C4]`
- [ ] Progress bar advances left to right
- [ ] Note counter increments
- [ ] Pressing 'q' exits early
- [ ] Terminal is restored to normal after exit

### 6. Quiet Mode Suppresses TUI
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/scale.ly --quiet
```
- [ ] No TUI appears — just plays audio and exits

## MIDI Output Tests

### 7. MIDI File Export
```
cargo run -p haydn-performer -- crates/haydn-performer/examples/scale.ly --synth midi --midi-out target/tmp/test.mid
```
- [ ] File `target/tmp/test.mid` is created
- [ ] File is a valid MIDI file (open in a DAW, MIDI player, or https://signal.vercel.app/edit)
- [ ] Notes correspond to C major scale

## Troubleshooting

**No audio at all:**
1. Check system volume isn't muted
2. Check default audio output device in OS sound settings
3. Try `cargo run -p haydn-performer -- --test-audio` for the quickest diagnostic
4. On Windows: some audio drivers need exclusive mode disabled

**Audio plays but sounds wrong:**
1. Clicks/pops → could be buffer underrun; try `--bpm 60` for longer notes
2. Wrong pitch → run `--test-audio` and compare to a piano (should be C-E-G = C major triad)
