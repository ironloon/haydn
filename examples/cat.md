# Cat

**Teaches:** Input/output — reading bytes and echoing them back.

Cat is the simplest I/O program: read a byte, print it, repeat until there's nothing left.  In Haydn, the `read` opcode pushes from the input stream (0 on EOF), and `loop_start`/`loop_end` give us the while-loop to keep going.

## The Program

```
C6  E4  C5  A5  C6  E4  D5  F4
```

Eight notes. Read, duplicate for the loop check, enter the loop, print, read next, duplicate, loop back, clean up.

## How It Works

```
Read          C6     Read byte (or 0 if empty)
Dup           E4     Copy for loop condition check
LoopStart     C5     Peek: nonzero → enter loop; zero → skip to end
  PrintChar   A5     Print the byte and consume it
  Read        C6     Read next byte
  Dup         E4     Copy for next loop check
LoopEnd       D5     Pop copy: nonzero → replay; zero → exit
Drop          F4     Clean up remaining stack value
```

The loop body records `[PrintChar, Read, Dup]` on the first pass. Each replay prints the current byte, reads the next, and copies it for the loop condition. When the input runs out, `read` returns 0, `dup` copies it, and the end-of-buffer check sees zero — the loop exits.

## Event Sequence

```
Op(Read), Op(Dup), Op(LoopStart),
  Op(PrintChar), Op(Read), Op(Dup),
Op(LoopEnd), Op(Drop)
```

Feed any input to the VM before starting. Every byte is echoed exactly once, in order. With no input, the loop is skipped entirely — `read` returns 0, `loop_start` sees zero on top, and execution jumps past `loop_end` to the final `drop`.
