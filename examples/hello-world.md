# Hello, World!

**Teaches:** Push values, arithmetic, print — the fundamental Haydn cycle.

Every Haydn program follows the same pattern: push values from the low register, operate with the high register, repeat. Hello World builds each ASCII character from pairs of negative numbers multiplied together, then prints the result. The performer's left hand supplies the raw material; the right hand shapes it into text.

## Piano Tuning Quick Reference

The default piano tuning divides the keyboard into two zones:

| Zone | Range | Function |
|------|-------|----------|
| **Value** | C2–B3 (MIDI 36–59) | `Push(note − 60)` — values −24 to −1 |
| **Operation** | C4+ (MIDI 60+) | White keys in C major → opcodes |

Key operation notes:

| Note | Opcode | Note | Opcode |
|------|--------|------|--------|
| C4 | `add` | C5 | `loop_start` |
| D4 | `sub` | D5 | `loop_end` |
| E4 | `dup` | A5 | `print_char` |
| F4 | `drop` | B5 | `print_num` |
| G4 | `mul` | C6 | `read` |

## The Program

Each character is built by multiplying two value-zone notes (producing the ASCII code), then printing with A5. Characters requiring values not directly available as products use an additional add or sub step.

```
             ┌─ Character ─┐
Note Sequence              ASCII  Output
─────────────────────────────────────────
E3  D#3 G4  A5              72    H
D3  D3  G4  B3  D4  A5     101    e
C3  D#3 G4  A5             108    l
C3  D#3 G4  A5             108    l
C3  D3  G4  D#3 C4  A5     111    o
G#3 C#3 G4  A5              44    ,
G#3 E3  G4  A5              32    (space)
D#3 D3  G4  A3  C4  A5      87    W
C3  D3  G4  D#3 C4  A5     111    o
F#3 F2  G4  A5             114    r
C3  D#3 G4  A5             108    l
D3  D3  G4  A5             100    d
A3  C#3 G4  A5              33    !
A#3 G3  G4  A5              10    \n
```

## Walkthrough

The table below traces the first three characters through the VM. Each row shows the note played, the event it produces, the stack state after execution, and any output.

### H (ASCII 72)

| # | Note | Event | Stack | Output |
|---|------|-------|-------|--------|
| 1 | E3 | `Push(-8)` | `[-8]` | |
| 2 | D#3 | `Push(-9)` | `[-8, -9]` | |
| 3 | G4 | `Op(Mul)` | `[72]` | |
| 4 | A5 | `Op(PrintChar)` | `[]` | `H` |

Two negative numbers multiplied: (−8) × (−9) = 72, the ASCII code for 'H'. PrintChar pops the value and writes the character.

### e (ASCII 101)

| # | Note | Event | Stack | Output |
|---|------|-------|-------|--------|
| 5 | D3 | `Push(-10)` | `[-10]` | |
| 6 | D3 | `Push(-10)` | `[-10, -10]` | |
| 7 | G4 | `Op(Mul)` | `[100]` | |
| 8 | B3 | `Push(-1)` | `[100, -1]` | |
| 9 | D4 | `Op(Sub)` | `[101]` | |
| 10 | A5 | `Op(PrintChar)` | `[]` | `e` |

(−10) × (−10) = 100. Then 100 − (−1) = 101. The sub opcode computes `a − b` where `b` is the top of stack, so subtracting −1 adds 1.

### l (ASCII 108)

| # | Note | Event | Stack | Output |
|---|------|-------|-------|--------|
| 11 | C3 | `Push(-12)` | `[-12]` | |
| 12 | D#3 | `Push(-9)` | `[-12, -9]` | |
| 13 | G4 | `Op(Mul)` | `[108]` | |
| 14 | A5 | `Op(PrintChar)` | `[]` | `l` |

(−12) × (−9) = 108. The second 'l' (steps 15–18) is identical.

## Event Sequence

The complete program as VM events, for reference:

```
// H (72): (-8) × (-9)
Push(-8), Push(-9), Mul, PrintChar,
// e (101): (-10) × (-10) - (-1) = 101
Push(-10), Push(-10), Mul, Push(-1), Sub, PrintChar,
// l (108): (-12) × (-9)
Push(-12), Push(-9), Mul, PrintChar,
// l (108)
Push(-12), Push(-9), Mul, PrintChar,
// o (111): (-12) × (-10) + (-9) = 111
Push(-12), Push(-10), Mul, Push(-9), Add, PrintChar,
// , (44): (-4) × (-11)
Push(-4), Push(-11), Mul, PrintChar,
// (space 32): (-4) × (-8)
Push(-4), Push(-8), Mul, PrintChar,
// W (87): (-9) × (-10) + (-3) = 87
Push(-9), Push(-10), Mul, Push(-3), Add, PrintChar,
// o (111): (-12) × (-10) + (-9)
Push(-12), Push(-10), Mul, Push(-9), Add, PrintChar,
// r (114): (-6) × (-19)
Push(-6), Push(-19), Mul, PrintChar,
// l (108): (-12) × (-9)
Push(-12), Push(-9), Mul, PrintChar,
// d (100): (-10) × (-10)
Push(-10), Push(-10), Mul, PrintChar,
// ! (33): (-3) × (-11)
Push(-3), Push(-11), Mul, PrintChar,
// \n (10): (-2) × (-5)
Push(-2), Push(-5), Mul, PrintChar,
```

## Arithmetic Reference

| Char | ASCII | Construction | Notes |
|------|-------|-------------|-------|
| H | 72 | (−8) × (−9) | Two pushes + multiply |
| e | 101 | (−10) × (−10) − (−1) | Product + subtract negative = add |
| l | 108 | (−12) × (−9) | Two pushes + multiply |
| o | 111 | (−12) × (−10) + (−9) | Product + add negative = subtract |
| , | 44 | (−4) × (−11) | Two pushes + multiply |
| ␣ | 32 | (−4) × (−8) | Two pushes + multiply |
| W | 87 | (−9) × (−10) + (−3) | Product + add negative |
| r | 114 | (−6) × (−19) | Reaches into octave 2 for F2 |
| d | 100 | (−10) × (−10) | Perfect square |
| ! | 33 | (−3) × (−11) | Two pushes + multiply |
| \n | 10 | (−2) × (−5) | Two pushes + multiply |

## Performance

Play these notes on any MIDI keyboard loaded with the default piano tuning. The low register (C2–B3) pushes values; the high register (C4+) triggers operations. Each character emerges as you strike the final A5 — watch the output appear letter by letter.

The entire program is 56 events: 22 push values and 34 operations (14 multiplies, 4 adds, 1 subtract, 14 print_chars). On a piano, it takes roughly 15 seconds at a comfortable tempo.
