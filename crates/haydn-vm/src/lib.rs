//! Haydn VM — virtual machine for the Haydn esoteric programming language.
//!
//! Haydn is an esoteric programming language where live musical performance IS the program.
//! This crate implements the stack-based, event-driven VM specified in the Haydn Language Specification.

mod types;
mod opcodes;
pub use types::*;

use std::collections::{HashMap, VecDeque};

pub struct HaydnVm {
    stack: Vec<i64>,
    memory: HashMap<i64, i64>,
    pub(crate) loop_state: LoopState,
    pub(crate) loop_stack: Vec<LoopFrame>,
    pub(crate) replay_stack: Vec<ReplayFrame>,
    input_buffer: VecDeque<u8>,
    output_buffer: Vec<u8>,
    pub(crate) event_queue: VecDeque<Event>,
    pub(crate) skip_depth: usize,
}

impl HaydnVm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            memory: HashMap::new(),
            loop_state: LoopState::Normal,
            loop_stack: Vec::new(),
            replay_stack: Vec::new(),
            input_buffer: VecDeque::new(),
            output_buffer: Vec::new(),
            event_queue: VecDeque::new(),
            skip_depth: 0,
        }
    }

    pub fn feed(&mut self, event: Event) {
        self.event_queue.push_back(event);
    }

    pub fn step(&mut self) -> Option<StepResult> {
        // If replaying, get next event from replay buffer
        if self.loop_state == LoopState::Replaying {
            if let Some(frame) = self.replay_stack.last_mut() {
                if frame.at_end() {
                    // End of buffer — check stack top to decide replay vs exit
                    let top = self.stack.last().copied().unwrap_or(0);
                    if top != 0 {
                        // Replay again
                        frame.reset();
                        let event = *frame.current()?;
                        return Some(self.execute_event(event));
                    } else {
                        // Exit loop
                        self.replay_stack.pop();
                        if !self.replay_stack.is_empty() {
                            self.loop_state = LoopState::Replaying;
                        } else if !self.loop_stack.is_empty() {
                            self.loop_state = LoopState::Recording;
                        } else {
                            self.loop_state = LoopState::Normal;
                        }
                        // Process queued events if back to normal
                        if self.loop_state == LoopState::Normal || self.loop_state == LoopState::Recording {
                            return self.step();
                        }
                        return None;
                    }
                } else {
                    let event = *frame.current()?;
                    frame.advance();
                    return Some(self.execute_event(event));
                }
            }
        }

        // If skipping, consume events looking for matching loop_end
        if self.skip_depth > 0 {
            let event = self.event_queue.pop_front()?;
            match event {
                Event::Op(Opcode::LoopStart) => {
                    self.skip_depth += 1;
                }
                Event::Op(Opcode::LoopEnd) => {
                    self.skip_depth -= 1;
                    if self.skip_depth == 0 {
                        // Back to previous state
                        if !self.loop_stack.is_empty() {
                            self.loop_state = LoopState::Recording;
                        } else {
                            self.loop_state = LoopState::Normal;
                        }
                    }
                }
                _ => {}
            }
            return Some(StepResult {
                event,
                operation: Operation::Noop,
                stack_snapshot: self.stack.clone(),
                output: None,
                edge_case: None,
            });
        }

        // Normal/recording: dequeue from event_queue
        let event = self.event_queue.pop_front()?;

        // If recording, append event to current loop buffer (except LoopEnd which is handled separately)
        if self.loop_state == LoopState::Recording {
            match event {
                Event::Op(Opcode::LoopEnd) => {
                    // Don't record loop_end — handle it specially
                }
                Event::Op(Opcode::LoopStart) => {
                    // Record it in parent buffer, then handle
                    if let Some(frame) = self.loop_stack.last_mut() {
                        frame.buffer.push(event);
                    }
                }
                _ => {
                    if let Some(frame) = self.loop_stack.last_mut() {
                        frame.buffer.push(event);
                    }
                }
            }
        }

        Some(self.execute_event(event))
    }

    pub fn process_event(&mut self, event: Event) -> Vec<StepResult> {
        self.feed(event);
        let mut results = Vec::new();
        while let Some(result) = self.step() {
            results.push(result);
        }
        results
    }

    pub fn provide_input(&mut self, bytes: &[u8]) {
        self.input_buffer.extend(bytes);
    }

    pub fn output(&self) -> &[u8] {
        &self.output_buffer
    }

    pub fn stack(&self) -> &[i64] {
        &self.stack
    }

    fn execute_event(&mut self, event: Event) -> StepResult {
        match event {
            Event::Push(value) => {
                self.stack.push(value);
                StepResult {
                    event,
                    operation: Operation::Pushed(value),
                    stack_snapshot: self.stack.clone(),
                    output: None,
                    edge_case: None,
                }
            }
            Event::Op(Opcode::LoopStart) | Event::Op(Opcode::LoopEnd) => {
                // Stub for Plan 02 — returns Noop
                StepResult {
                    event,
                    operation: Operation::Noop,
                    stack_snapshot: self.stack.clone(),
                    output: None,
                    edge_case: None,
                }
            }
            Event::Op(opcode) => {
                let (operation, output, edge_case) = opcodes::execute_opcode(self, opcode);
                StepResult {
                    event,
                    operation,
                    stack_snapshot: self.stack.clone(),
                    output,
                    edge_case,
                }
            }
        }
    }
}

impl Default for HaydnVm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_has_19_variants() {
        // Verify all 19 opcodes exist and are distinct
        let opcodes = [
            Opcode::Dup, Opcode::Swap, Opcode::Drop, Opcode::Rotate,
            Opcode::Add, Opcode::Sub, Opcode::Mul, Opcode::Div, Opcode::Mod,
            Opcode::Eq, Opcode::Gt, Opcode::Lt,
            Opcode::LoopStart, Opcode::LoopEnd,
            Opcode::PrintNum, Opcode::PrintChar, Opcode::Read,
            Opcode::Store, Opcode::Load,
        ];
        assert_eq!(opcodes.len(), 19);
        for (i, a) in opcodes.iter().enumerate() {
            for (j, b) in opcodes.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn test_event_push_creates_push_event() {
        let e = Event::Push(42);
        assert_eq!(e, Event::Push(42));
    }

    #[test]
    fn test_event_op_creates_op_event() {
        let e = Event::Op(Opcode::Add);
        assert_eq!(e, Event::Op(Opcode::Add));
    }

    #[test]
    fn test_event_and_opcode_derive_traits() {
        // Clone
        let e = Event::Push(1);
        let e2 = e;
        assert_eq!(e, e2);

        let o = Opcode::Dup;
        let o2 = o;
        assert_eq!(o, o2);
    }

    #[test]
    fn test_vm_new_creates_empty_vm() {
        let vm = HaydnVm::new();
        assert!(vm.stack().is_empty());
        assert!(vm.output().is_empty());
        assert_eq!(vm.loop_state, LoopState::Normal);
    }

    #[test]
    fn test_feed_then_step_push() {
        let mut vm = HaydnVm::new();
        vm.feed(Event::Push(5));
        let result = vm.step().unwrap();
        assert_eq!(result.stack_snapshot, vec![5]);
        assert_eq!(result.operation, Operation::Pushed(5));
        assert_eq!(result.event, Event::Push(5));
    }

    #[test]
    fn test_step_on_idle_returns_none() {
        let mut vm = HaydnVm::new();
        assert!(vm.step().is_none());
    }

    #[test]
    fn test_process_event_push() {
        let mut vm = HaydnVm::new();
        let results = vm.process_event(Event::Push(10));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].stack_snapshot, vec![10]);
    }

    #[test]
    fn test_provide_input() {
        let mut vm = HaydnVm::new();
        vm.provide_input(b"abc");
        assert_eq!(vm.input_buffer.len(), 3);
        assert_eq!(vm.input_buffer[0], b'a');
        assert_eq!(vm.input_buffer[1], b'b');
        assert_eq!(vm.input_buffer[2], b'c');
    }

    #[test]
    fn test_step_result_fields() {
        let mut vm = HaydnVm::new();
        let results = vm.process_event(Event::Push(42));
        let r = &results[0];
        assert_eq!(r.event, Event::Push(42));
        assert_eq!(r.operation, Operation::Pushed(42));
        assert_eq!(r.stack_snapshot, vec![42]);
        assert!(r.output.is_none());
        assert!(r.edge_case.is_none());
    }

    #[test]
    fn test_multiple_pushes() {
        let mut vm = HaydnVm::new();
        vm.process_event(Event::Push(1));
        vm.process_event(Event::Push(2));
        let results = vm.process_event(Event::Push(3));
        assert_eq!(results[0].stack_snapshot, vec![1, 2, 3]);
    }
}
