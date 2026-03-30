use std::sync::mpsc;

pub mod display;

pub use haydn_tuning;
pub use haydn_vm;

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

pub fn note_name(midi: u8) -> String {
    let name = NOTE_NAMES[(midi % 12) as usize];
    let octave = (midi as i16 / 12) - 1;
    format!("{}{}", name, octave)
}

#[derive(Debug)]
pub enum MidiMsg {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
}

pub fn midi_callback(_timestamp: u64, message: &[u8], tx: &mut mpsc::Sender<MidiMsg>) {
    if let Ok(event) = midly::live::LiveEvent::parse(message) {
        if let midly::live::LiveEvent::Midi { message: msg, .. } = event {
            match msg {
                midly::MidiMessage::NoteOn { key, vel } => {
                    if vel.as_int() == 0 {
                        let _ = tx.send(MidiMsg::NoteOff { note: key.as_int() });
                    } else {
                        let _ = tx.send(MidiMsg::NoteOn {
                            note: key.as_int(),
                            velocity: vel.as_int(),
                        });
                    }
                }
                midly::MidiMessage::NoteOff { key, .. } => {
                    let _ = tx.send(MidiMsg::NoteOff { note: key.as_int() });
                }
                _ => {}
            }
        }
    }
}

pub fn format_event_log(note: u8, velocity: u8, result: &haydn_vm::StepResult) -> String {
    use haydn_vm::{Event, Operation};

    let mapping = match &result.operation {
        Operation::Pushed(v) => format!("Push({})", v),
        Operation::Executed(op) => format!("Op({:?})", op),
        Operation::LoopEntered => "LoopEntered".to_string(),
        Operation::LoopSkipped => "LoopSkipped".to_string(),
        Operation::LoopExited => "LoopExited".to_string(),
        Operation::LoopReplaying => "LoopReplaying".to_string(),
        Operation::ReplayStep(evt) => match evt {
            Event::Push(v) => format!("Replay(Push({}))", v),
            Event::Op(op) => format!("Replay(Op({:?}))", op),
        },
        Operation::EndOfBufferReplay => "EndOfBufferReplay".to_string(),
        Operation::EndOfBufferExit => "EndOfBufferExit".to_string(),
        Operation::Noop => "Noop".to_string(),
    };

    let stack = format!("{:?}", result.stack_snapshot);
    let mut line = format!(
        "[{} v={}] → {}  |  Stack: {}",
        note_name(note),
        velocity,
        mapping,
        stack
    );

    if let Some(ref output) = result.output {
        if !output.is_empty() {
            let text = String::from_utf8_lossy(output);
            line.push_str(&format!("  |  Out: \"{}\"", text));
        }
    }

    if let Some(ref ec) = result.edge_case {
        line.push_str(&format!("  |  ⚠ {:?}", ec));
    }

    line
}

pub fn format_session_summary(vm: &haydn_vm::HaydnVm) -> String {
    let mut lines = Vec::new();
    lines.push("─── Session Summary ───".to_string());
    lines.push(format!(
        "Final stack ({} items): {:?}",
        vm.stack().len(),
        vm.stack()
    ));
    if !vm.output().is_empty() {
        match std::str::from_utf8(vm.output()) {
            Ok(s) => lines.push(format!("Program output: {}", s)),
            Err(_) => lines.push(format!("Program output (bytes): {:?}", vm.output())),
        }
    }
    lines.push("───────────────────────".to_string());
    lines.join("\n")
}

/// Process a single note-on event through the tuning engine and VM.
/// Returns formatted log lines, or a single "(unmapped)" line.
pub fn process_note(
    note: u8,
    velocity: u8,
    engine: &mut haydn_tuning::TuningEngine,
    vm: &mut haydn_vm::HaydnVm,
) -> Vec<String> {
    match engine.map_note(note) {
        Some(event) => {
            let results = vm.process_event(event);
            results
                .iter()
                .map(|r| format_event_log(note, velocity, r))
                .collect()
        }
        None => {
            vec![format!("[{} v={}] → (unmapped)", note_name(note), velocity)]
        }
    }
}
