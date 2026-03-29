use haydn_vm::*;

// === Helpers ===

fn run(events: &[Event]) -> (HaydnVm, Vec<StepResult>) {
    let mut vm = HaydnVm::new();
    let mut results = Vec::new();
    for &event in events {
        results.extend(vm.process_event(event));
    }
    (vm, results)
}

fn run_with_input(events: &[Event], input: &[u8]) -> (HaydnVm, Vec<StepResult>) {
    let mut vm = HaydnVm::new();
    vm.provide_input(input);
    let mut results = Vec::new();
    for &event in events {
        results.extend(vm.process_event(event));
    }
    (vm, results)
}

/// Compile a Brainfuck program to a Haydn event sequence (spec §7.3-7.4).
fn bf_compile(program: &str) -> Vec<Event> {
    // Setup: store 1 at memory[0] (data pointer = cell 0, which is memory[1])
    let mut events = vec![
        Event::Push(1),
        Event::Push(0),
        Event::Op(Opcode::Store),
    ];

    for ch in program.chars() {
        match ch {
            '>' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Push(1), Event::Op(Opcode::Add),
                Event::Push(0), Event::Op(Opcode::Store),
            ]),
            '<' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Push(1), Event::Op(Opcode::Sub),
                Event::Push(0), Event::Op(Opcode::Store),
            ]),
            '+' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Dup), Event::Op(Opcode::Load),
                Event::Push(1), Event::Op(Opcode::Add),
                Event::Op(Opcode::Swap), Event::Op(Opcode::Store),
            ]),
            '-' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Dup), Event::Op(Opcode::Load),
                Event::Push(1), Event::Op(Opcode::Sub),
                Event::Op(Opcode::Swap), Event::Op(Opcode::Store),
            ]),
            '.' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Load), Event::Op(Opcode::PrintChar),
            ]),
            ',' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Read),
                Event::Op(Opcode::Swap), Event::Op(Opcode::Store),
            ]),
            '[' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Load), Event::Op(Opcode::LoopStart),
            ]),
            ']' => events.extend_from_slice(&[
                Event::Push(0), Event::Op(Opcode::Load),
                Event::Op(Opcode::Load), Event::Op(Opcode::LoopEnd),
                Event::Op(Opcode::Drop),
            ]),
            _ => {} // Ignore non-BF characters
        }
    }
    events
}

// === Spec §8.1: Hello ===

#[test]
fn test_spec_hello() {
    let (vm, _) = run(&[
        Event::Push(72), Event::Op(Opcode::PrintChar),
        Event::Push(101), Event::Op(Opcode::PrintChar),
        Event::Push(108), Event::Op(Opcode::PrintChar),
        Event::Push(108), Event::Op(Opcode::PrintChar),
        Event::Push(111), Event::Op(Opcode::PrintChar),
    ]);
    assert_eq!(vm.output(), b"Hello");
}

// === Spec §8.2: Arithmetic ===

#[test]
fn test_spec_arithmetic() {
    let (vm, _) = run(&[
        Event::Push(3),
        Event::Push(4),
        Event::Op(Opcode::Add),
        Event::Op(Opcode::PrintNum),
    ]);
    assert_eq!(vm.output(), b"7");
}

// === Spec §8.3: Countdown ===

#[test]
fn test_spec_countdown() {
    let (vm, _) = run(&[
        Event::Push(5),
        Event::Op(Opcode::LoopStart),
        Event::Op(Opcode::Dup),
        Event::Op(Opcode::PrintNum),
        Event::Push(10),
        Event::Op(Opcode::PrintChar),
        Event::Push(1),
        Event::Op(Opcode::Sub),
        Event::Op(Opcode::Dup),
        Event::Op(Opcode::LoopEnd),
        Event::Op(Opcode::Drop),
    ]);
    assert_eq!(vm.output(), b"5\n4\n3\n2\n1\n");
}

// === BF Compiler Tests ===

#[test]
fn test_bf_compile_empty() {
    let events = bf_compile("");
    // Setup only: Push(1), Push(0), Op(Store)
    assert_eq!(events.len(), 3);
}

#[test]
fn test_bf_zero_cell() {
    // Pre-populate cell 0 with 3 via "+++", then zero it with "[-]"
    let events = bf_compile("+++[-]");
    let (vm, _) = run(&events);
    // Cell 0 (memory[1]) should be 0
    // Verify by loading cell value
    let mut vm = vm;
    let results = vm.process_event(Event::Push(1));
    let _ = results;
    let results = vm.process_event(Event::Op(Opcode::Load));
    assert_eq!(*results[0].stack_snapshot.last().unwrap(), 0);
}

#[test]
fn test_bf_cat_one_char() {
    // BF ",+." with input "A" → output "B" (65 + 1 = 66)
    let events = bf_compile(",+.");
    let (vm, _) = run_with_input(&events, b"A");
    assert_eq!(vm.output(), b"B");
}

#[test]
fn test_bf_cat_echo() {
    // BF ",." with input "X" → output "X"
    let events = bf_compile(",.");
    let (vm, _) = run_with_input(&events, b"X");
    assert_eq!(vm.output(), b"X");
}

#[test]
fn test_bf_add_two_cells() {
    // BF: put 3 in cell 0, 5 in cell 1, add cell 0 to cell 1, print cell 1
    // "+++>+++++<[->+<]>."
    // Cell 0 = 3, Cell 1 = 5, move cell 0 to cell 1 → cell 1 = 8, print char 8
    let events = bf_compile("+++>+++++<[->+<]>.");
    let (vm, _) = run(&events);
    assert_eq!(vm.output(), &[8u8]); // ASCII 8 (not printable, but correct)
}

#[test]
fn test_bf_hello_world() {
    // Classic BF Hello World (Wikipedia canonical version)
    let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let events = bf_compile(program);
    let (vm, _) = run(&events);
    assert_eq!(vm.output(), b"Hello World!\n");
}

#[test]
fn test_bf_simple_loop() {
    // BF "+++++[.-]" — print chars 5, 4, 3, 2, 1 then stop
    let events = bf_compile("+++++[.-]");
    let (vm, _) = run(&events);
    assert_eq!(vm.output().len(), 5);
    assert_eq!(vm.output()[0], 5u8);
    assert_eq!(vm.output()[4], 1u8);
}

#[test]
fn test_bf_nested_loop() {
    // BF: multiply 3 * 4 = 12 using nested loops
    // "+++[>++++<-]>." — cell 0 = 3, inner adds 4 to cell 1 each iteration, cell 1 = 12
    let events = bf_compile("+++[>++++<-]>.");
    let (vm, _) = run(&events);
    assert_eq!(vm.output(), &[12u8]);
}

#[test]
fn test_bf_skip_empty_loop() {
    // BF "[]." on empty cell → loop skipped, then print cell 0 = 0
    let events = bf_compile("[].");
    let (vm, _) = run(&events);
    assert_eq!(vm.output(), &[0u8]);
}

// === Edge Case Integration Tests ===

#[test]
fn test_many_pushes() {
    let events: Vec<Event> = (0..50).map(|i| Event::Push(i)).collect();
    let (vm, _) = run(&events);
    assert_eq!(vm.stack().len(), 50);
}

#[test]
fn test_push_drop_cycle() {
    let mut events = Vec::new();
    for i in 0..100 {
        events.push(Event::Push(i));
        events.push(Event::Op(Opcode::Drop));
    }
    let (vm, _) = run(&events);
    assert!(vm.stack().is_empty());
}

#[test]
fn test_memory_roundtrip() {
    let addresses = [0i64, 1, 100, 999];
    let mut events = Vec::new();
    for (i, &addr) in addresses.iter().enumerate() {
        // Store value (i+1)*10 at address
        events.push(Event::Push((i as i64 + 1) * 10));
        events.push(Event::Push(addr));
        events.push(Event::Op(Opcode::Store));
    }
    // Load them all back
    for &addr in &addresses {
        events.push(Event::Push(addr));
        events.push(Event::Op(Opcode::Load));
    }
    let (vm, _) = run(&events);
    assert_eq!(vm.stack(), &[10, 20, 30, 40]);
}

#[test]
fn test_no_panic_on_chaos() {
    // A mix of 200+ events including edge cases — should never panic
    let mut events = Vec::new();
    for i in 0..50 {
        events.push(Event::Push(i % 3));
        events.push(Event::Op(Opcode::Dup));
        events.push(Event::Op(Opcode::Add));
        events.push(Event::Op(Opcode::Drop));
    }
    // Underflow-inducing ops
    for _ in 0..20 {
        events.push(Event::Op(Opcode::Drop));
        events.push(Event::Op(Opcode::Add));
        events.push(Event::Op(Opcode::Sub));
    }
    // Unmatched loop ends
    for _ in 0..5 {
        events.push(Event::Op(Opcode::LoopEnd));
    }
    // Division by zero
    events.push(Event::Push(10));
    events.push(Event::Push(0));
    events.push(Event::Op(Opcode::Div));
    events.push(Event::Push(0));
    events.push(Event::Op(Opcode::Mod));
    // Store/load with negative address
    events.push(Event::Push(42));
    events.push(Event::Push(-1));
    events.push(Event::Op(Opcode::Store));
    events.push(Event::Push(-1));
    events.push(Event::Op(Opcode::Load));
    // Print operations
    events.push(Event::Push(65));
    events.push(Event::Op(Opcode::PrintChar));
    events.push(Event::Push(42));
    events.push(Event::Op(Opcode::PrintNum));
    // Read with no input
    events.push(Event::Op(Opcode::Read));

    // This must not panic
    let (_, results) = run(&events);
    assert!(results.len() > 200);
}

#[test]
fn test_all_opcodes_exist_in_events() {
    // Verify every opcode can be wrapped in an Event and processed
    let opcodes = [
        Opcode::Dup, Opcode::Swap, Opcode::Drop, Opcode::Rotate,
        Opcode::Add, Opcode::Sub, Opcode::Mul, Opcode::Div, Opcode::Mod,
        Opcode::Eq, Opcode::Gt, Opcode::Lt,
        Opcode::LoopStart, Opcode::LoopEnd,
        Opcode::PrintNum, Opcode::PrintChar, Opcode::Read,
        Opcode::Store, Opcode::Load,
    ];
    let mut vm = HaydnVm::new();
    // Push enough values so ops don't underflow
    for i in 0..20 {
        vm.process_event(Event::Push(i + 1));
    }
    for opcode in opcodes {
        let results = vm.process_event(Event::Op(opcode));
        assert!(!results.is_empty(), "Opcode {:?} produced no result", opcode);
    }
}

#[test]
fn test_process_event_returns_all_replay_steps() {
    // A loop that replays should return all replay results from process_event
    let mut vm = HaydnVm::new();
    vm.process_event(Event::Push(2));
    vm.process_event(Event::Op(Opcode::LoopStart));
    vm.process_event(Event::Push(1));
    vm.process_event(Event::Op(Opcode::Sub));
    vm.process_event(Event::Op(Opcode::Dup));
    // LoopEnd triggers replay — should return all replay results in one call
    let results = vm.process_event(Event::Op(Opcode::LoopEnd));
    // Should have: LoopEnd(replaying) + replay steps + end-of-buffer results
    assert!(results.len() > 1);
}
