# Phase 2: VM Core — Research

**Researched:** 2026-03-28
**Discovery Level:** 1 (Quick Verification — pure Rust, zero external deps, well-specified)
**Status:** Complete

## Research Question

"What do I need to know to PLAN the VM Core implementation well?"

The answer: the spec is exhaustive. This research translates the Haydn Language Specification into Rust implementation guidance, identifies the tricky spots, and recommends module/type/test structure.

## 1. VM Architecture

### 1.1 Core Struct

The `HaydnVm` struct holds the complete VM state per spec §2.5:

```rust
pub struct HaydnVm {
    stack: Vec<i64>,
    memory: HashMap<i64, i64>,      // Sparse — only stores non-zero values
    loop_stack: Vec<LoopFrame>,     // Stack of loop recording buffers
    loop_state: LoopState,          // Normal | Recording | Replaying
    input_buffer: VecDeque<u8>,     // For read opcode (D-08)
    output_buffer: Vec<u8>,         // Accumulated output bytes
    event_queue: VecDeque<Event>,   // Live events queued during replay (spec §5.4 rule 4)
    replay_stack: Vec<ReplayFrame>, // Position tracking for nested replay
}
```

**Why `HashMap<i64, i64>` for memory?** Spec says memory is addressable by non-negative `i64` — we can't allocate 2^63 slots up front. Sparse map with default 0 for missing keys is the natural fit. Negative address reads return 0, writes are no-ops (spec §2.3).

**Why `VecDeque<u8>` for input?** D-08 specifies `provide_input(bytes)` with FIFO semantics and `read` pops from front. VecDeque gives O(1) push_back and pop_front.

### 1.2 Step API (D-01, D-02)

Per CONTEXT.md decisions D-01 and D-02:

```rust
// D-01: step() processes exactly one event
pub fn step(&mut self, event: Event) -> StepResult { ... }

// D-02: process_event() convenience wrapper
pub fn process_event(&mut self, event: Event) -> Vec<StepResult> { ... }
```

`step()` is the primitive. During loop replay, a single live `Op(loop_end)` might trigger a full buffer replay — but each replayed event is one step. The caller calls `step()` once with the live event, and internally the VM may process replayed events. The question is: does `step()` return multiple results (one per replayed event) or just one?

**Recommendation:** `step()` processes exactly ONE event (live or replayed) and returns ONE `StepResult`. When `loop_end` triggers replay, the caller must keep calling `step_continue()` (or similar) to drain the replay. This gives the TUI (Phase 5) granular control over replay pacing per D-03.

Alternative: `step()` could accept the live event and return a `Vec<StepResult>` for all replay steps it triggers. But this conflicts with D-03's requirement that the caller controls replay pacing.

**Chosen approach:**
```rust
/// Process next pending step (replay or queued event). Returns None if VM is idle.
pub fn step(&mut self) -> Option<StepResult> { ... }

/// Feed a live event into the VM.
pub fn feed(&mut self, event: Event) { ... }

/// Feed + drain all steps (convenience per D-02)
pub fn process_event(&mut self, event: Event) -> Vec<StepResult> {
    self.feed(event);
    let mut results = Vec::new();
    while let Some(result) = self.step() {
        results.push(result);
    }
    results
}
```

This separates event feeding from stepping. The TUI can call `feed()` then `step()` one at a time with delays for visible replay. The test harness uses `process_event()` for simplicity.

### 1.3 StepResult (D-01, D-04, D-07)

```rust
pub struct StepResult {
    pub event: Event,                   // The event that was processed
    pub operation: Operation,           // What the VM did (for TUI display)
    pub stack_snapshot: Vec<i64>,       // Stack state after this step
    pub output: Option<Vec<u8>>,        // Output bytes (D-07), None for non-output steps
    pub edge_case: Option<EdgeCase>,    // D-04: optional edge case flag
}
```

**Stack snapshot vs delta:** Full snapshot is simpler for v1 and what the TUI needs. A snapshot of a Vec<i64> is a clone — for stacks under ~1000 elements this is negligible. If profiling shows issues later, switch to delta.

### 1.4 Edge Cases (D-04)

```rust
pub enum EdgeCase {
    StackUnderflow,       // Operation needed more elements than available
    DivisionByZero,       // div or mod with zero divisor
    ModuloByZero,         // mod with zero divisor
    NegativeAddress,      // store/load with addr < 0
    MaxLoopDepthExceeded, // loop_start at depth 256
    UnmatchedLoopEnd,     // loop_end with no active loop
    EmptyStackDefault,    // Operation used default 0 for missing stack value
}
```

The VM executes gracefully in ALL cases per spec §3.2. The EdgeCase enum is purely informational for the TUI (D-04).

## 2. Type System

### 2.1 Opcode Enum

Direct from spec §3.4:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Dup, Swap, Drop, Rotate,
    Add, Sub, Mul, Div, Mod,
    Eq, Gt, Lt,
    LoopStart, LoopEnd,
    PrintNum, PrintChar, Read,
    Store, Load,
}
```

19 variants, one per opcode. Derive `Copy` — opcodes are small value types.

### 2.2 Event Enum

Per spec §4.2:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Push(i64),
    Op(Opcode),
}
```

`Clone` needed for loop recording (events get stored in buffers). Don't derive `Copy` because the enum is 16 bytes (discriminant + i64) — borderline for Copy, but cloning explicitly is fine.

Actually, 16 bytes is perfectly fine for Copy in Rust. Let's derive Copy for Event too — it simplifies buffer operations.

### 2.3 Operation Enum (for StepResult)

What the VM "did" — useful for TUI display:

```rust
pub enum Operation {
    Pushed(i64),              // Push(v) → value pushed
    Executed(Opcode),         // Op executed normally
    LoopEntered,              // loop_start → entered recording
    LoopSkipped,              // loop_start → skipped (zero/empty)
    LoopExited,               // loop_end → exited (zero)
    LoopReplaying,            // loop_end → started replay (nonzero)
    ReplayStep(Event),        // Processing a replayed event
    EndOfBufferReplay,        // End of buffer → replay again
    EndOfBufferExit,          // End of buffer → exit loop
    Noop,                     // No-op (e.g., drop on empty, unmatched loop_end)
}
```

## 3. Loop State Machine

This is the most complex part. The spec §5 defines it precisely.

### 3.1 Data Structures

```rust
enum LoopState {
    Normal,
    Recording,
    Replaying,
}

struct LoopFrame {
    buffer: Vec<Event>,     // Recorded events for this loop level
}

struct ReplayFrame {
    buffer: Vec<Event>,     // The buffer being replayed
    position: usize,        // Current position in buffer
}
```

The `loop_stack` holds `LoopFrame`s during recording. When replay starts, the top frame moves to `replay_stack` as a `ReplayFrame`.

### 3.2 Key Implementation Details

**loop_start (Normal state):**
1. Peek stack top (don't pop). Empty stack → treat as 0.
2. If zero → must skip to matching loop_end. Scan incoming live events, tracking nesting depth. This is tricky in an event-driven model — we can't "skip ahead" because events arrive one at a time.
3. If nonzero → push new `LoopFrame` onto loop_stack, transition to Recording.

**The skip problem:** In a stored-program VM, `[` skips to matching `]` by scanning the program. In Haydn's event-driven model, events arrive one at a time. When `loop_start` decides to skip, the VM must enter a "skipping" state and discard incoming events until the matching `loop_end` arrives. We need a skip counter:

```rust
enum LoopState {
    Normal,
    Recording,
    Replaying,
    Skipping(usize),  // depth counter — 0 means next loop_end exits skip
}
```

When `loop_start` skips: enter `Skipping(0)`. In Skipping state:
- `Op(loop_start)` → increment depth
- `Op(loop_end)` → if depth > 0, decrement; if depth == 0, return to previous state (Normal or Recording)
- All other events → discard (no-op)

**This is critical and easy to get wrong.** The skip must respect nesting.

**loop_end (Recording state):**
1. Pop stack top. Empty → treat as 0.
2. Nonzero → pop LoopFrame from loop_stack, convert to ReplayFrame at position 0, push onto replay_stack, transition to Replaying.
3. Zero → discard LoopFrame, pop loop_stack. If loop_stack empty → Normal. Else → stay Recording (nested loop exited back to outer recording).

**End of replay buffer:**
1. Pop stack top. Empty → treat as 0.
2. Nonzero → reset position to 0, replay again.
3. Zero → discard ReplayFrame. If replay_stack has more frames → continue outer replay (Replaying). If loop_stack has frames → Recording (was replaying inner loop inside a recording). Else → Normal.

**Nested loops during replay:**
When replaying and hit `loop_start` with nonzero top → push new LoopFrame and start Recording (within the replay context). This is the Replaying→Recording transition from the spec table.

### 3.3 Event Queuing During Replay

Spec §5.4 rule 4: "Live input is queued during replay." When in Replaying state, new live events go into `event_queue`. After replay completes, drain the queue.

With the `feed()`/`step()` API this is natural: `feed()` adds to queue when replaying, `step()` processes from replay buffer first, then queued events.

### 3.4 Stream End

Spec §5.4 rule 8: If the event stream ends while loops are open, discard all loop buffers, return to Normal. Implement as a `fn close(&mut self)` or `fn reset_loops(&mut self)`.

## 4. Opcode Implementation Patterns

### 4.1 Binary Operations Pattern

All binary ops (add, sub, mul, div, mod, eq, gt, lt) follow the same pattern:

```rust
fn binary_op(&mut self, f: impl Fn(i64, i64) -> i64) -> Option<EdgeCase> {
    let b = self.stack.pop().unwrap_or(0);  // right operand (top)
    let a = self.stack.pop().unwrap_or(0);  // left operand (second)
    // If either was missing, flag edge case
    self.stack.push(f(a, b));
    // Return edge case if stack was insufficient
}
```

Edge case tracking: if the stack had fewer than 2 elements, return `EdgeCase::StackUnderflow` (or `EmptyStackDefault`) in the StepResult.

**Wrapping arithmetic:** Use Rust's `i64::wrapping_add()`, `wrapping_sub()`, `wrapping_mul()`. For div/mod by zero, push 0.

**Division toward zero:** Rust's `/` operator already truncates toward zero for integers. ✓

**Modulo sign follows dividend:** Rust's `%` operator already produces a result with the sign of the dividend. ✓

### 4.2 Stack Operations

- **dup:** Peek top, push copy. Empty → push 0. (Not `pop+push+push` — it's duplicate, so peek.)
- **swap:** Swap last two. Fewer than 2 → no-op.
- **drop:** Pop top. Empty → no-op.
- **rotate:** `a b c → b c a` (third element moves to top). Fewer than 3 → no-op. Implementation: remove element at `len-3`, push to top.

### 4.3 Memory Operations

- **store:** Pop addr (top), pop val (second). If addr < 0 → no-op. Else `memory.insert(addr, val)`. Fewer than 2 → missing values are 0.
- **load:** Pop addr. If addr < 0 → push 0. Else push `memory.get(&addr).copied().unwrap_or(0)`. Empty stack → push `memory[0]` (load from address 0 per spec).

### 4.4 I/O Operations

- **print_num:** Pop top (or 0), format as decimal string, write bytes to output buffer.
- **print_char:** Pop top. Empty → no-op (spec: "do not print NUL"). If value exists, compute `(value % 256) as u8`, write to output buffer.
- **read:** Push one byte from input_buffer, or 0 if empty.

## 5. Module Organization

Recommended structure for `crates/haydn-vm/src/`:

```
lib.rs          — Public API: re-exports, HaydnVm struct, step()/feed()/process_event()
types.rs        — Opcode, Event, StepResult, EdgeCase, Operation enums/structs
opcodes.rs      — Opcode execution implementations (binary_op, stack_ops, memory_ops, io_ops)
loop_machine.rs — Loop state machine (LoopState, LoopFrame, ReplayFrame, skip logic)
```

Keep `lib.rs` as the public interface. Internal modules are `pub(crate)`. Export only what downstream crates need: `HaydnVm`, `Event`, `Opcode`, `StepResult`, `EdgeCase`, `Operation`.

**Alternative:** Single `lib.rs` file. Given the VM is ~500-800 lines total, this is viable. But splitting aids readability and lets us test modules independently.

**Recommendation:** Start with split modules. The loop machine alone is ~200 lines and benefits from isolation.

## 6. Testing Strategy

### 6.1 Unit Tests per Opcode

Each opcode gets dedicated tests covering:
1. Normal operation (sufficient stack elements)
2. Edge case (insufficient stack, division by zero, etc.)
3. Wrapping behavior (overflow/underflow for arithmetic)

### 6.2 Loop State Machine Tests

Critical test scenarios:
1. Simple loop (countdown from N to 0)
2. Skip path (loop_start with 0 top)
3. Nested loops (inner loop completes, outer continues)
4. Unmatched loop_end (should be no-op)
5. Max nesting depth (256th loop_start is no-op)
6. Empty stack at loop_start/loop_end boundaries
7. Live events queued during replay
8. Stream end with open loops

### 6.3 Integration Tests (Spec Examples)

The spec provides three worked examples (§8):
1. Hello — push and print chars
2. Arithmetic — add two numbers, print_num
3. Countdown loop — loop pedal with decrementing counter

Plus the Turing completeness proof's worked example:
4. BF `[-]` (zero a cell) via the Haydn reduction

These should be integration tests that drive the VM with event sequences and assert output.

### 6.4 Brainfuck Compilation Tests

Implement a simple `bf_compile(program: &str) -> Vec<Event>` test helper that translates BF to Haydn events per spec §7.4. Then run classic BF programs:
- `[-]` — zero current cell
- `[->+<]` — copy cell
- `,+.` — read, increment, print (cat-ish)
- Hello World — full BF hello world

This validates the TC proof is implementable and the loop machine handles real-world nesting.

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Loop skip in event-driven model is tricky | Medium | High | Add `Skipping(depth)` state, extensive nesting tests |
| Nested loop + replay interaction bugs | Medium | High | Trace-level assertions in tests, compare against spec step traces |
| `step()` API design — feed/step vs step-with-event | Low | Medium | Research resolved: feed()/step() separation (§1.2) |
| Memory HashMap performance with large addresses | Low | Low | Acceptable for v1; switch to BTreeMap or segmented array if needed |
| Stack snapshot cloning in StepResult | Low | Low | Negligible for stacks < 1000; optimize if profiling shows issues |

## 8. What NOT to Do

- **Don't use `tokio` or async** — VM is pure synchronous computation. Zero need for async.
- **Don't add external dependencies** — `haydn-vm` should have zero deps (per D-05). Only std.
- **Don't implement event metadata** — v1 ignores it (spec §4.4). Don't parse, store, or process metadata.
- **Don't add timing/tempo** — VM has no internal clock (spec §6.2).
- **Don't use `panic!` ANYWHERE in VM code** — spec §3.2 says the VM never panics. All errors are graceful returns.

## 9. Validation Architecture

### Test Categories and Acceptance Criteria

**Category A: Opcode Correctness (VM-01, VM-02, VM-03, VM-04, VM-06, VM-07)**
- Every opcode has at least 2 tests: normal case + edge case
- Arithmetic opcodes test wrapping behavior
- Memory opcodes test negative addresses
- I/O opcodes test empty buffer behavior

**Category B: Loop Machine (VM-05)**
- Simple loop, nested loop, skip path, max depth
- Event queuing during replay
- Unmatched boundaries
- All spec §5.3 transition table rows exercised

**Category C: Edge Case Handling (VM-08)**
- Empty stack for every opcode
- Division/modulo by zero
- Negative memory addresses
- Max loop depth (256)
- Stream end with open loops

**Category D: Integration (VM-01 combined)**
- Spec §8 examples produce correct output
- BF compilation + execution produces correct output

### Nyquist Validation Criteria

Each test category maps to requirements:
- Category A → VM-01, VM-02, VM-03, VM-04, VM-06, VM-07
- Category B → VM-05
- Category C → VM-08
- Category D → VM-01 (integration validation)

## Sources

- `docs/SPECIFICATION.md` — Haydn Language Specification v1.0 (primary source)
- `.planning/phases/02-vm-core/02-CONTEXT.md` — User decisions D-01 through D-13
- `.planning/phases/01-language-specification/01-CONTEXT.md` — Phase 1 foundational decisions
- Rust `std::collections::HashMap` — for sparse memory addressing
- Rust `std::collections::VecDeque` — for FIFO input buffer
- Rust wrapping arithmetic: `i64::wrapping_add()`, `wrapping_sub()`, `wrapping_mul()`

## RESEARCH COMPLETE

**Recommendation:** Proceed to planning. This is a Level 1 research phase — the spec is exhaustive, zero external deps needed. Key insights:
1. `feed()`/`step()` API separation gives TUI replay pacing control (D-03)
2. `Skipping(depth)` loop state is critical for event-driven skip-to-matching-end
3. Split into 4 modules: lib, types, opcodes, loop_machine
4. Test via spec examples + BF compilation for integration coverage
