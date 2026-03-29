use std::collections::HashSet;

use crate::error::{TuningError, ValidationResult};
use crate::types::{parse_opcode, RawEmit, RawTrigger, RawTuningFile};

/// Validate a parsed tuning file, collecting all errors in a single pass.
pub fn validate(raw: &RawTuningFile) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate values section
    if let Some(ref values) = raw.values {
        if values.range[0] > values.range[1] {
            errors.push(TuningError::Validation {
                section: "values".to_string(),
                message: format!(
                    "range[0] ({}) must be <= range[1] ({})",
                    values.range[0], values.range[1]
                ),
            });
        }
        if values.range[1] > 127 {
            errors.push(TuningError::NoteOutOfRange {
                note: values.range[1],
            });
        }
    }

    // Validate operations section
    if let Some(ref operations) = raw.operations {
        for (key, opcode_name) in operations {
            match key.parse::<u8>() {
                Ok(note) => {
                    if note > 127 {
                        errors.push(TuningError::NoteOutOfRange { note });
                    }
                }
                Err(_) => {
                    errors.push(TuningError::Validation {
                        section: "operations".to_string(),
                        message: format!("key '{key}' is not a valid MIDI note number (0-127)"),
                    });
                }
            }
            if let Err(e) = parse_opcode(opcode_name) {
                errors.push(e);
            }
        }
    }

    // Validate rules
    for (i, rule) in raw.rules.iter().enumerate() {
        let rule_label = format!("rules[{i}]");
        match &rule.trigger {
            RawTrigger::Note { note } => {
                if *note > 127 {
                    errors.push(TuningError::NoteOutOfRange { note: *note });
                }
            }
            RawTrigger::Range { from, to } => {
                if from > to {
                    errors.push(TuningError::Validation {
                        section: rule_label.clone(),
                        message: format!("range from ({from}) must be <= to ({to})"),
                    });
                }
                if *to > 127 {
                    errors.push(TuningError::NoteOutOfRange { note: *to });
                }
            }
            RawTrigger::Interval { .. } => {}
        }
        if let RawEmit::Op { ref opcode } = rule.emit {
            if let Err(e) = parse_opcode(opcode) {
                errors.push(e);
            }
        }
    }

    // Check for overlapping exact-note rules
    check_overlaps(raw, &mut errors);

    // Coverage warnings
    if !raw.metadata.partial_coverage {
        check_coverage(raw, &mut warnings);
    }

    ValidationResult { errors, warnings }
}

/// Check if two rules can match the same note (exact note and range overlaps).
fn check_overlaps(raw: &RawTuningFile, errors: &mut Vec<TuningError>) {
    // Collect all exact notes and ranges into a single list of (note, label) pairs
    let mut note_sources: Vec<(u8, String)> = Vec::new();

    // From operations shorthand
    if let Some(ref ops) = raw.operations {
        for key in ops.keys() {
            if let Ok(note) = key.parse::<u8>() {
                note_sources.push((note, format!("operations[{key}]")));
            }
        }
    }

    // From exact note rules
    for (i, rule) in raw.rules.iter().enumerate() {
        if let RawTrigger::Note { note } = &rule.trigger {
            note_sources.push((*note, format!("rules[{i}]")));
        }
    }

    // Check for duplicates
    let mut seen: std::collections::HashMap<u8, String> = std::collections::HashMap::new();
    for (note, label) in &note_sources {
        if let Some(prev_label) = seen.get(note) {
            errors.push(TuningError::Overlap {
                rule_a: prev_label.clone(),
                rule_b: label.clone(),
                note: *note,
            });
        } else {
            seen.insert(*note, label.clone());
        }
    }

    // Check if operations notes fall within values range
    if let Some(ref values) = raw.values {
        let range_label = format!("values[{}-{}]", values.range[0], values.range[1]);
        for (note, label) in &note_sources {
            if *note >= values.range[0] && *note <= values.range[1] {
                errors.push(TuningError::Overlap {
                    rule_a: range_label.clone(),
                    rule_b: label.clone(),
                    note: *note,
                });
            }
        }
    }
}

/// Check if all 19 opcodes are covered.
fn check_coverage(raw: &RawTuningFile, warnings: &mut Vec<String>) {
    let mut mapped_opcodes: HashSet<String> = HashSet::new();

    if let Some(ref ops) = raw.operations {
        for opcode_name in ops.values() {
            mapped_opcodes.insert(opcode_name.trim().to_lowercase());
        }
    }

    for rule in &raw.rules {
        if let RawEmit::Op { ref opcode } = rule.emit {
            mapped_opcodes.insert(opcode.trim().to_lowercase());
        }
    }

    let all_opcodes = [
        "dup", "swap", "drop", "rotate", "add", "sub", "mul", "div", "mod", "eq", "gt", "lt",
        "loop_start", "loop_end", "print_num", "print_char", "read", "store", "load",
    ];

    let missing: Vec<&str> = all_opcodes
        .iter()
        .filter(|op| !mapped_opcodes.contains(**op))
        .copied()
        .collect();

    if !missing.is_empty() {
        warnings.push(format!(
            "unmapped opcodes (set partial_coverage = true to suppress): {}",
            missing.join(", ")
        ));
    }
}
