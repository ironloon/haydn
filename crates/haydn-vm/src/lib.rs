//! Haydn VM — virtual machine for the Haydn esoteric programming language.
//!
//! Haydn is an esoteric programming language where live musical performance IS the program.
//! This crate implements the stack-based, event-driven VM specified in the Haydn Language Specification.

mod types;
mod opcodes;
mod loop_machine;
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
        // Phase 1: Check replay buffer end-of-buffer
        if let Some(frame) = self.replay_stack.last() {
            if frame.at_end() {
                return Some(loop_machine::handle_end_of_buffer(self));
            }
        }

        // Phase 2: Get next event from source
        let event = if let Some(frame) = self.replay_stack.last_mut() {
            // Reading from replay buffer
            let e = *frame.current()?;
            frame.advance();
            e
        } else {
            // Normal/Recording/Skip: from event queue
            self.event_queue.pop_front()?
        };

        // Phase 3: Handle skip mode
        if self.skip_depth > 0 {
            return Some(loop_machine::handle_skip(self, event));
        }

        // Phase 4: Handle loop opcodes
        match event {
            Event::Op(Opcode::LoopStart) => {
                return Some(loop_machine::handle_loop_start(self, event));
            }
            Event::Op(Opcode::LoopEnd) => {
                return Some(loop_machine::handle_loop_end(self, event));
            }
            _ => {}
        }

        // Phase 5: Record in active loop frames (skip parents of active replay)
        let base = self.recording_base();
        for frame in &mut self.loop_stack[base..] {
            frame.buffer.push(event);
        }

        // Phase 6: Execute
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

    pub fn close(&mut self) {
        self.loop_stack.clear();
        self.replay_stack.clear();
        self.loop_state = LoopState::Normal;
        self.skip_depth = 0;
    }

    /// Returns the loop_stack index below which recording should be skipped.
    /// During replay, events should only be recorded in loop frames created
    /// *after* the current replay started, not in pre-existing parent frames
    /// that already contain those events from the original recording pass.
    pub(crate) fn recording_base(&self) -> usize {
        self.replay_stack.last().map(|f| f.parent_loop_depth).unwrap_or(0)
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
                // Should not reach here — handled in step() via loop_machine
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
