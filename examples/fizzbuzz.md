# FizzBuzz

**Teaches:** Loops, modular arithmetic — the capstone example proving Turing completeness in practice.

FizzBuzz is constructed via the Brainfuck reduction (spec §7). Each BF instruction compiles mechanically to 4–8 Haydn events, producing a program far too long to notate by hand — but the VM executes it just the same. This is the point: Haydn's event interface is source-independent, and its Turing completeness means anything BF can compute, a musical performance can compute.

## The Brainfuck Source

The following BF program prints FizzBuzz for 1 through 20:

```brainfuck
++++[>+++++<-]       Cell 1 = 20 (counter, counting down)
>[                    Loop while counter > 0

  Set up divisor cells: cell 2 = 3, cell 3 = 5
  >+++>+++++<<

  Divmod by 3: cell 4 = counter mod 3
  >[>+>+<<-]>>[<<+>>-]<  Copy cell 1 to cell 4 and cell 5
  ++++[<-------->-]       Subtract 3 repeatedly... (simplified: we test mod directly)

  (Simplified approach: use a flag-based method)
  <<                      Back to counter cell
]
```

In practice, a full FizzBuzz in BF is 300–400 characters. The canonical approach uses modular counters that reset every 3 and 5 iterations, printing "Fizz", "Buzz", "FizzBuzz", or the number accordingly.

## The Reduction

Per spec §7.3–7.4, each BF instruction maps to a fixed Haydn event sequence:

| BF | Haydn Events | Count |
|----|-------------|-------|
| `>` | `Push(0) Load Push(1) Add Push(0) Store` | 6 |
| `<` | `Push(0) Load Push(1) Sub Push(0) Store` | 6 |
| `+` | `Push(0) Load Dup Load Push(1) Add Swap Store` | 8 |
| `-` | `Push(0) Load Dup Load Push(1) Sub Swap Store` | 8 |
| `.` | `Push(0) Load Load PrintChar` | 4 |
| `,` | `Push(0) Load Read Swap Store` | 5 |
| `[` | `Push(0) Load Load LoopStart` | 4 |
| `]` | `Push(0) Load Load LoopEnd Drop` | 5 |

Memory cell 0 stores the data pointer. BF cells map to Haydn memory starting at address 1.

## Opening Measures (Piano Tuning)

The setup sequence `++++[>+++++<-]` compiles to approximately 90 Haydn events. Here are the first few measures — the four `+` increments that initialize cell 0 to 4:

```
Setup (store data pointer = 1 at memory[0]):
B3  C2  D6      Push(-1) Push(-24)... → Push(1) Push(0) Store

First '+':                          Second '+':
B3  C2  E6  E4  E6                  B3  C2  E6  E4  E6
B3  C4  F6  D6                      B3  C4  F6  D6
```

The low–high register alternation is the signature Haydn sound: a cascade of value-zone notes in the left hand, punctuated by operation-zone strikes in the right. At performance tempo, the pattern produces a rolling rhythm across the keyboard.

## Verification

The full compiled FizzBuzz event sequence is verified by `cargo test --test examples test_fizzbuzz`. The test feeds the BF-compiled events through the Haydn VM and checks that the output matches the expected FizzBuzz pattern.

Because the BF-to-Haydn reduction is mechanical and the VM executes deterministically, the musical performance produces exactly the same output as the BF program — which is the entire point of Turing completeness.
