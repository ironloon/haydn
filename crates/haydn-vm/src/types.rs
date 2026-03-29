/// All 19 opcodes in the Haydn instruction set (spec §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    Dup,
    Swap,
    Drop,
    Rotate,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Gt,
    Lt,
    LoopStart,
    LoopEnd,
    PrintNum,
    PrintChar,
    Read,
    Store,
    Load,
}

/// Events are the only way to drive the VM (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Push(i64),
    Op(Opcode),
}

/// Edge cases that are flagged but never cause panics (spec §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeCase {
    StackUnderflow,
    DivisionByZero,
    ModuloByZero,
    NegativeAddress,
    MaxLoopDepthExceeded,
    UnmatchedLoopEnd,
    EmptyStackDefault,
}

/// What the VM did in response to an event — for TUI display and testing.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Pushed(i64),
    Executed(Opcode),
    LoopEntered,
    LoopSkipped,
    LoopExited,
    LoopReplaying,
    ReplayStep(Event),
    EndOfBufferReplay,
    EndOfBufferExit,
    Noop,
}

/// Result of processing a single event/step.
#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    pub event: Event,
    pub operation: Operation,
    pub stack_snapshot: Vec<i64>,
    pub output: Option<Vec<u8>>,
    pub edge_case: Option<EdgeCase>,
}

/// Internal loop state tracking (spec §5.2).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LoopState {
    Normal,
    Recording,
    Replaying,
}

/// A recording buffer for one loop level.
#[derive(Debug, Clone)]
pub(crate) struct LoopFrame {
    pub(crate) buffer: Vec<Event>,
}

/// A replay position within a recorded buffer.
#[derive(Debug, Clone)]
pub(crate) struct ReplayFrame {
    pub(crate) buffer: Vec<Event>,
    pub(crate) position: usize,
    pub(crate) parent_loop_depth: usize,
}

impl ReplayFrame {
    pub(crate) fn current(&self) -> Option<&Event> {
        self.buffer.get(self.position)
    }

    pub(crate) fn advance(&mut self) {
        self.position += 1;
    }

    pub(crate) fn reset(&mut self) {
        self.position = 0;
    }

    pub(crate) fn at_end(&self) -> bool {
        self.position >= self.buffer.len()
    }
}
