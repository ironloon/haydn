# Truth Machine

**Teaches:** Conditional loops — branching on input.

The truth machine is a classic esolang benchmark: read one character. If it's `0`, print `0` and halt. If it's `1`, print `1` forever. In Haydn, the loop pedal model handles both cases — `loop_start` peeks the stack and either enters recording or skips to the end.

## The Program

```
C6  F#3 E3  G4  D4  E4  F#3 E3  G4  C4  A5  C5  E4  F3  F3  G4  A5  D5
```

18 notes. Read the input, convert from ASCII to digit, print the digit, then loop (or not).

## How It Works

```
Read              C6       Read input byte ('0'=48 or '1'=49)
Push(-6)          F#3  ┐
Push(-8)          E3   │  Build 48 (ASCII '0')
Mul               G4   ┘  (-6) × (-8) = 48
Sub               D4       input - 48 → digit (0 or 1)
Dup               E4       Copy digit for printing
Push(-6)          F#3  ┐
Push(-8)          E3   │  Build 48 again
Mul               G4   ┘  (-6) × (-8) = 48
Add               C4       digit + 48 → ASCII character
PrintChar         A5       Print '0' or '1'. Stack: [digit]
LoopStart         C5       Peek digit: 0 → skip, 1 → enter loop
  Dup             E4       [1, 1]
  Push(-7)        F3   ┐
  Push(-7)        F3   │  Build 49 (ASCII '1')
  Mul             G4   ┘  (-7) × (-7) = 49
  PrintChar       A5       Print '1'. Stack: [1, 1]
LoopEnd           D5       Pop 1 → nonzero → replay forever
```

**Input '0':** digit = 0. After printing '0', `loop_start` peeks 0 and skips the loop body. Output: `0`. Program ends.

**Input '1':** digit = 1. After printing '1', `loop_start` peeks 1 and enters the loop. The loop body prints '1' and leaves 1 on the stack. `loop_end` pops the duplicate (1 ≠ 0), so the buffer replays — forever. Output: `111111111...`

## Event Sequence

```
// Read and convert to digit
Op(Read),
Push(-6), Push(-8), Op(Mul),   // 48
Op(Sub),                        // input - 48
// Print digit as ASCII
Op(Dup),
Push(-6), Push(-8), Op(Mul),   // 48
Op(Add),                        // digit + 48
Op(PrintChar),                  // print '0' or '1'
// Infinite loop for '1'
Op(LoopStart),
  Op(Dup),
  Push(-7), Push(-7), Op(Mul), // 49
  Op(PrintChar),                // print '1'
Op(LoopEnd)
```
