use crate::error::TuningError;
use crate::types::{parse_opcode, Metadata, RawEmit, RawTrigger, RawTuningFile};
use crate::validate::validate;

/// A compiled trigger condition.
#[derive(Debug)]
enum Trigger {
    ExactNote(u8),
    NoteRange { from: u8, to: u8 },
    Interval(i8),
}

/// What to produce when a trigger matches.
#[derive(Debug)]
enum Action {
    Push(i64),
    PushOffset { root_note: u8 },
    Op(haydn_vm::Opcode),
}

/// A single compiled rule: trigger → action.
#[derive(Debug)]
struct CompiledRule {
    trigger: Trigger,
    action: Action,
}

/// The tuning engine: compiles a tuning file into a fast rule matcher.
///
/// Evaluation order: value range → operations (sorted by note) → raw rules.
/// First match wins.
pub struct TuningEngine {
    rules: Vec<CompiledRule>,
    root_note: u8,
    metadata: Metadata,
    last_note: Option<u8>,
}

impl TuningEngine {
    /// Compile a `RawTuningFile` into a `TuningEngine`.
    /// Returns all validation errors if the file is invalid.
    pub fn compile(raw: RawTuningFile) -> Result<TuningEngine, Vec<TuningError>> {
        let result = validate(&raw);
        if !result.errors.is_empty() {
            return Err(result.errors);
        }

        let root_note = raw.metadata.root_note;
        let metadata = raw.metadata.clone();
        let mut rules = Vec::new();

        // 1. Values shorthand → NoteRange with PushOffset
        if let Some(ref values) = raw.values {
            rules.push(CompiledRule {
                trigger: Trigger::NoteRange {
                    from: values.range[0],
                    to: values.range[1],
                },
                action: Action::PushOffset { root_note },
            });
        }

        // 2. Operations shorthand → ExactNote with Op, sorted by note ascending
        if let Some(ref operations) = raw.operations {
            let mut ops: Vec<(u8, haydn_vm::Opcode)> = Vec::new();
            for (key, opcode_name) in operations {
                let note: u8 = key.parse().expect("validated");
                let opcode = parse_opcode(opcode_name).expect("validated");
                ops.push((note, opcode));
            }
            ops.sort_by_key(|(note, _)| *note);
            for (note, opcode) in ops {
                rules.push(CompiledRule {
                    trigger: Trigger::ExactNote(note),
                    action: Action::Op(opcode),
                });
            }
        }

        // 3. Raw rules in array order
        for raw_rule in &raw.rules {
            let trigger = match &raw_rule.trigger {
                RawTrigger::Note { note } => Trigger::ExactNote(*note),
                RawTrigger::Range { from, to } => Trigger::NoteRange {
                    from: *from,
                    to: *to,
                },
                RawTrigger::Interval { semitones } => Trigger::Interval(*semitones),
            };
            let action = match &raw_rule.emit {
                RawEmit::Push { value } => Action::Push(*value),
                RawEmit::PushOffset => Action::PushOffset { root_note },
                RawEmit::Op { opcode } => {
                    let op = parse_opcode(opcode).expect("validated");
                    Action::Op(op)
                }
            };
            rules.push(CompiledRule { trigger, action });
        }

        Ok(TuningEngine {
            rules,
            root_note,
            metadata,
            last_note: None,
        })
    }

    /// Map a MIDI note to a VM event. First matching rule wins.
    /// Updates internal interval tracking regardless of match.
    pub fn map_note(&mut self, midi_note: u8) -> Option<haydn_vm::Event> {
        let result = self.rules.iter().find_map(|rule| {
            let matched = match &rule.trigger {
                Trigger::ExactNote(n) => midi_note == *n,
                Trigger::NoteRange { from, to } => midi_note >= *from && midi_note <= *to,
                Trigger::Interval(semitones) => {
                    if let Some(last) = self.last_note {
                        (midi_note as i16 - last as i16) == *semitones as i16
                    } else {
                        false
                    }
                }
            };
            if matched {
                Some(match &rule.action {
                    Action::Push(v) => haydn_vm::Event::Push(*v),
                    Action::PushOffset { root_note } => {
                        haydn_vm::Event::Push(midi_note as i64 - *root_note as i64)
                    }
                    Action::Op(opcode) => haydn_vm::Event::Op(*opcode),
                })
            } else {
                None
            }
        });

        // Always update last_note for interval tracking
        self.last_note = Some(midi_note);
        result
    }

    /// Get the tuning's metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get the root note.
    pub fn root_note(&self) -> u8 {
        self.root_note
    }

    /// Reset interval tracking state.
    pub fn reset(&mut self) {
        self.last_note = None;
    }
}
