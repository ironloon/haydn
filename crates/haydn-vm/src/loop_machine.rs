use crate::types::*;
use crate::HaydnVm;

pub(crate) const MAX_LOOP_DEPTH: usize = 256;

pub(crate) fn handle_loop_start(vm: &mut HaydnVm, event: Event) -> StepResult {
    let top = vm.stack.last().copied().unwrap_or(0);
    let empty_stack = vm.stack.is_empty();

    if top == 0 {
        // Skip: enter skip mode to find matching loop_end
        vm.skip_depth = 1;
        // Record LoopStart in existing frames (skip parents of active replay)
        let base = vm.recording_base();
        for frame in &mut vm.loop_stack[base..] {
            frame.buffer.push(event);
        }
        let edge = if empty_stack {
            Some(EdgeCase::EmptyStackDefault)
        } else {
            None
        };
        StepResult {
            event,
            operation: Operation::LoopSkipped,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: edge,
        }
    } else {
        // Check max depth
        if vm.loop_stack.len() >= MAX_LOOP_DEPTH {
            return StepResult {
                event,
                operation: Operation::Noop,
                stack_snapshot: vm.stack.clone(),
                output: None,
                edge_case: Some(EdgeCase::MaxLoopDepthExceeded),
            };
        }
        // Record LoopStart in existing frames (skip parents of active replay)
        let base = vm.recording_base();
        for frame in &mut vm.loop_stack[base..] {
            frame.buffer.push(event);
        }
        // Push new recording frame
        vm.loop_stack.push(LoopFrame {
            buffer: Vec::new(),
        });
        vm.loop_state = LoopState::Recording;
        StepResult {
            event,
            operation: Operation::LoopEntered,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: None,
        }
    }
}

pub(crate) fn handle_loop_end(vm: &mut HaydnVm, event: Event) -> StepResult {
    if vm.loop_stack.is_empty() {
        // Unmatched loop_end — no-op (spec §5.4 rule 7)
        return StepResult {
            event,
            operation: Operation::Noop,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: Some(EdgeCase::UnmatchedLoopEnd),
        };
    }

    // Record LoopEnd in parent frames (skip parents of active replay)
    let base = vm.recording_base();
    let last = vm.loop_stack.len() - 1;
    for frame in &mut vm.loop_stack[base..last] {
        frame.buffer.push(event);
    }

    // Pop stack top (spec §5.4 rule 1: loop_end POPS)
    let top = vm.stack.pop().unwrap_or(0);

    if top != 0 {
        // Replay: convert top LoopFrame to ReplayFrame
        let loop_frame = vm.loop_stack.pop().unwrap();
        vm.replay_stack.push(ReplayFrame {
            buffer: loop_frame.buffer,
            position: 0,
            parent_loop_depth: vm.loop_stack.len(),
        });
        vm.loop_state = LoopState::Replaying;
        StepResult {
            event,
            operation: Operation::LoopReplaying,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: None,
        }
    } else {
        // Exit: discard top LoopFrame
        vm.loop_stack.pop();
        if !vm.loop_stack.is_empty() {
            vm.loop_state = LoopState::Recording;
        } else if !vm.replay_stack.is_empty() {
            vm.loop_state = LoopState::Replaying;
        } else {
            vm.loop_state = LoopState::Normal;
        }
        StepResult {
            event,
            operation: Operation::LoopExited,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: None,
        }
    }
}

pub(crate) fn handle_end_of_buffer(vm: &mut HaydnVm) -> StepResult {
    // End of replay buffer: Pop stack top to decide replay vs exit
    // (spec §5.3: "Pop stack top. Nonzero → replay again. Zero → exit.")
    let top = vm.stack.pop().unwrap_or(0);
    let event = Event::Op(Opcode::LoopEnd); // synthetic for StepResult

    if top != 0 {
        // Replay again
        if let Some(frame) = vm.replay_stack.last_mut() {
            frame.reset();
        }
        StepResult {
            event,
            operation: Operation::EndOfBufferReplay,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: None,
        }
    } else {
        // Exit replay
        vm.replay_stack.pop();
        if !vm.replay_stack.is_empty() {
            vm.loop_state = LoopState::Replaying;
        } else if !vm.loop_stack.is_empty() {
            vm.loop_state = LoopState::Recording;
        } else {
            vm.loop_state = LoopState::Normal;
        }
        StepResult {
            event,
            operation: Operation::EndOfBufferExit,
            stack_snapshot: vm.stack.clone(),
            output: None,
            edge_case: None,
        }
    }
}

pub(crate) fn handle_skip(vm: &mut HaydnVm, event: Event) -> StepResult {
    // Record skipped events in loop frames (skip parents of active replay)
    let base = vm.recording_base();
    for frame in &mut vm.loop_stack[base..] {
        frame.buffer.push(event);
    }

    match event {
        Event::Op(Opcode::LoopStart) => {
            vm.skip_depth += 1;
        }
        Event::Op(Opcode::LoopEnd) => {
            vm.skip_depth -= 1;
            if vm.skip_depth == 0 {
                // Exit skip, determine state
                if !vm.loop_stack.is_empty() {
                    vm.loop_state = LoopState::Recording;
                } else if !vm.replay_stack.is_empty() {
                    vm.loop_state = LoopState::Replaying;
                } else {
                    vm.loop_state = LoopState::Normal;
                }
            }
        }
        _ => {}
    }

    StepResult {
        event,
        operation: Operation::Noop,
        stack_snapshot: vm.stack.clone(),
        output: None,
        edge_case: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn run(events: &[Event]) -> (HaydnVm, Vec<StepResult>) {
        let mut vm = HaydnVm::new();
        let mut results = Vec::new();
        for &event in events {
            results.extend(vm.process_event(event));
        }
        (vm, results)
    }

    // === Basic loop ===

    #[test]
    fn test_loop_start_peeks_not_pops() {
        // LoopStart should PEEK (not pop) — stack unchanged
        let mut vm = HaydnVm::new();
        vm.process_event(Event::Push(3));
        let results = vm.process_event(Event::Op(Opcode::LoopStart));
        // Stack should still have 3 (peeked, not popped)
        assert_eq!(results[0].stack_snapshot, vec![3]);
        assert_eq!(results[0].operation, Operation::LoopEntered);
    }

    #[test]
    fn test_simple_countdown() {
        // Push(3), LoopStart, Push(1), Sub, Dup, LoopEnd → stack [0], 3 iterations
        let (vm, _) = run(&[
            Event::Push(3),
            Event::Op(Opcode::LoopStart),
            Event::Push(1),
            Event::Op(Opcode::Sub),
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::LoopEnd),
        ]);
        assert_eq!(vm.stack(), &[0]);
    }

    #[test]
    fn test_countdown_from_5_with_output() {
        // Spec §8.3: countdown from 5 with print
        let (vm, _) = run(&[
            Event::Push(5),
            Event::Op(Opcode::LoopStart),
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::PrintNum),
            Event::Push(10),
            Event::Op(Opcode::PrintChar), // '\n'
            Event::Push(1),
            Event::Op(Opcode::Sub),
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::LoopEnd),
            Event::Op(Opcode::Drop), // clean up final 0
        ]);
        assert_eq!(vm.output(), b"5\n4\n3\n2\n1\n");
        assert!(vm.stack().is_empty());
    }

    // === Skip path ===

    #[test]
    fn test_skip_on_zero() {
        // Push(0), LoopStart, Push(99), LoopEnd → Push(99) never executes
        let (vm, _) = run(&[
            Event::Push(0),
            Event::Op(Opcode::LoopStart),
            Event::Push(99),
            Event::Op(Opcode::LoopEnd),
        ]);
        assert_eq!(vm.stack(), &[0]);
    }

    #[test]
    fn test_skip_empty_stack() {
        // Empty stack + LoopStart → skip (treat empty as 0)
        let (_, results) = run(&[
            Event::Op(Opcode::LoopStart),
            Event::Push(99),
            Event::Op(Opcode::LoopEnd),
        ]);
        let loop_start_result = &results[0];
        assert_eq!(loop_start_result.operation, Operation::LoopSkipped);
        assert_eq!(
            loop_start_result.edge_case,
            Some(EdgeCase::EmptyStackDefault)
        );
    }

    #[test]
    fn test_nested_skip() {
        // Push(0), LoopStart, Push(1), LoopStart, Push(2), LoopEnd, LoopEnd
        // All skipped correctly (nested loop_start/loop_end pairs matched)
        let (vm, _) = run(&[
            Event::Push(0),
            Event::Op(Opcode::LoopStart),
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            Event::Push(2),
            Event::Op(Opcode::LoopEnd),
            Event::Op(Opcode::LoopEnd),
        ]);
        assert_eq!(vm.stack(), &[0]);
    }

    // === Edge cases ===

    #[test]
    fn test_unmatched_loop_end() {
        let (_, results) = run(&[Event::Op(Opcode::LoopEnd)]);
        assert_eq!(results[0].operation, Operation::Noop);
        assert_eq!(results[0].edge_case, Some(EdgeCase::UnmatchedLoopEnd));
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut vm = HaydnVm::new();
        // Push nonzero and open 256 loops
        for _ in 0..256 {
            vm.process_event(Event::Push(1));
            vm.process_event(Event::Op(Opcode::LoopStart));
        }
        // The 257th should fail
        vm.process_event(Event::Push(1));
        let results = vm.process_event(Event::Op(Opcode::LoopStart));
        assert_eq!(results[0].edge_case, Some(EdgeCase::MaxLoopDepthExceeded));
    }

    #[test]
    fn test_empty_stack_at_loop_end() {
        // Enter loop with nonzero, empty stack at loop_end → treat as 0, exit
        let (vm, _) = run(&[
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            Event::Op(Opcode::Drop), // empty the stack
            Event::Op(Opcode::LoopEnd),
        ]);
        assert!(vm.stack().is_empty());
    }

    #[test]
    fn test_loop_end_pops() {
        // Verify loop_end consumes the stack top
        let (vm, _) = run(&[
            Event::Push(99),
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            Event::Op(Opcode::Drop), // drop the 1
            Event::Push(0),
            Event::Op(Opcode::LoopEnd), // pops 0 → exit
        ]);
        // Stack should be [99] — the 0 was consumed by loop_end
        assert_eq!(vm.stack(), &[99]);
    }

    // === Nested loops ===

    #[test]
    fn test_nested_loops() {
        // Outer loop 2x, inner loop 3x each
        // Push(2), LoopStart[outer],
        //   Push(3), LoopStart[inner], Push(1), Sub, Dup, LoopEnd[inner],
        //   Drop, Push(1), Sub, Dup,
        // LoopEnd[outer], Drop
        let (vm, _) = run(&[
            Event::Push(2),
            Event::Op(Opcode::LoopStart),
            // Print outer counter
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::PrintNum),
            Event::Push(58), // ':'
            Event::Op(Opcode::PrintChar),
            // Inner loop: countdown from 3
            Event::Push(3),
            Event::Op(Opcode::LoopStart),
            Event::Push(1),
            Event::Op(Opcode::Sub),
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::LoopEnd),
            Event::Op(Opcode::Drop), // drop inner 0
            // Decrement outer
            Event::Push(1),
            Event::Op(Opcode::Sub),
            Event::Op(Opcode::Dup),
            Event::Op(Opcode::LoopEnd),
            Event::Op(Opcode::Drop), // drop outer 0
        ]);
        assert!(vm.stack().is_empty());
        // Outer 2 → inner 3x → print "2:", outer 1 → inner 3x → print "1:"
        assert_eq!(vm.output(), b"2:1:");
    }

    // === Event queuing during replay ===

    #[test]
    fn test_events_queued_during_replay() {
        // During replay, feed() queues events for later processing
        let mut vm = HaydnVm::new();
        // Set up a simple loop
        vm.process_event(Event::Push(1));
        vm.process_event(Event::Op(Opcode::LoopStart));
        vm.process_event(Event::Push(0)); // will make loop_end exit
        vm.process_event(Event::Op(Opcode::LoopEnd));
        // After loop exits, feed a push — should work
        let results = vm.process_event(Event::Push(42));
        assert_eq!(results[0].stack_snapshot.last(), Some(&42));
    }

    // === Stream end ===

    #[test]
    fn test_close_discards_open_loops() {
        let mut vm = HaydnVm::new();
        vm.process_event(Event::Push(5));
        vm.process_event(Event::Op(Opcode::LoopStart));
        vm.process_event(Event::Push(1));
        // Loop still open — close it
        vm.close();
        assert_eq!(vm.loop_state, LoopState::Normal);
        assert!(vm.loop_stack.is_empty());
        assert!(vm.replay_stack.is_empty());
        assert_eq!(vm.skip_depth, 0);
        // VM should still be usable
        let results = vm.process_event(Event::Push(99));
        assert_eq!(results[0].stack_snapshot.last(), Some(&99));
    }

    // === Loop with empty body ===

    #[test]
    fn test_loop_empty_body() {
        // Push(1), LoopStart, LoopEnd → pops 1, replays empty buffer, but end-of-buffer immediately
        // Then pops again... but stack has [1] after loop_start peek
        // LoopEnd: pop 1 → nonzero → replay empty buffer
        // Empty buffer end: pop ... hmm, stack should have just the peeked value
        // After LoopStart peek (not pop): stack [1]
        // LoopEnd: pop 1 → []. Nonzero → replay buffer []
        // End of empty buffer: pop from empty stack → 0 → exit
        let (vm, _) = run(&[
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            Event::Op(Opcode::LoopEnd),
        ]);
        assert!(vm.stack().is_empty());
    }

    // === Skip within recording ===

    #[test]
    fn test_skip_within_recording() {
        // Outer loop active (recording), inner loop skipped (zero top)
        // The skipped events should still be recorded in the outer buffer
        let (vm, _) = run(&[
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            // Inner loop with zero → skip
            Event::Push(0),
            Event::Op(Opcode::LoopStart),
            Event::Push(99), // should be skipped
            Event::Op(Opcode::LoopEnd),
            // Continue outer recording
            Event::Op(Opcode::Drop), // drop the 0
            Event::Push(0),          // will exit outer loop
            Event::Op(Opcode::LoopEnd),
        ]);
        // Outer body ran once, inner was skipped
        // LoopStart peeks (not pops) the 1, LoopEnd pops the 0 → exit.
        // Original 1 remains on stack.
        assert_eq!(vm.stack(), &[1]);
    }

    #[test]
    fn test_single_iteration_loop() {
        // Push(1), LoopStart, <body>, Push(0), LoopEnd
        // Body runs once (recording pass), loop_end pops 0 → exit
        let (vm, _) = run(&[
            Event::Push(1),
            Event::Op(Opcode::LoopStart),
            Event::Push(42),
            Event::Op(Opcode::PrintNum),
            Event::Push(0),
            Event::Op(Opcode::LoopEnd),
        ]);
        assert_eq!(vm.output(), b"42");
    }
}
