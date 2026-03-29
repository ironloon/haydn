use haydn_tuning::types::parse_opcode;
use haydn_tuning::RawTuningFile;
use haydn_tuning::validate::validate;
use haydn_vm::Opcode;

// ── Deserialization tests ──────────────────────────────────────────

#[test]
fn minimal_metadata_only() {
    let toml = r#"
        [metadata]
        name = "Test"
        root_note = 60
        instrument = "piano"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    assert_eq!(raw.metadata.name, "Test");
    assert_eq!(raw.metadata.root_note, 60);
    assert_eq!(raw.metadata.instrument, "piano");
    assert!(raw.metadata.author.is_none());
    assert!(!raw.metadata.partial_coverage);
    assert!(raw.values.is_none());
    assert!(raw.operations.is_none());
    assert!(raw.rules.is_empty());
}

#[test]
fn full_toml_all_fields() {
    let toml = r#"
        [metadata]
        name = "Full Test"
        root_note = 60
        instrument = "piano"
        author = "Test Author"
        partial_coverage = true

        [values]
        range = [36, 59]

        [operations]
        60 = "add"
        62 = "sub"

        [[rules]]
        [rules.trigger]
        type = "note"
        note = 64

        [rules.emit]
        type = "op"
        opcode = "dup"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    assert_eq!(raw.metadata.name, "Full Test");
    assert_eq!(raw.metadata.author.as_deref(), Some("Test Author"));
    assert!(raw.metadata.partial_coverage);
    assert!(raw.values.is_some());
    let values = raw.values.as_ref().unwrap();
    assert_eq!(values.range, [36, 59]);
    assert!(raw.operations.is_some());
    let ops = raw.operations.as_ref().unwrap();
    assert_eq!(ops.len(), 2);
    assert_eq!(ops.get("60").unwrap(), "add");
    assert_eq!(raw.rules.len(), 1);
}

#[test]
fn missing_metadata_fails() {
    let toml = r#"
        [values]
        range = [36, 59]
    "#;
    let result = toml::from_str::<RawTuningFile>(toml);
    assert!(result.is_err());
}

// ── Opcode parsing tests ──────────────────────────────────────────

#[test]
fn parse_all_19_opcodes() {
    let cases = [
        ("dup", Opcode::Dup),
        ("swap", Opcode::Swap),
        ("drop", Opcode::Drop),
        ("rotate", Opcode::Rotate),
        ("add", Opcode::Add),
        ("sub", Opcode::Sub),
        ("mul", Opcode::Mul),
        ("div", Opcode::Div),
        ("mod", Opcode::Mod),
        ("eq", Opcode::Eq),
        ("gt", Opcode::Gt),
        ("lt", Opcode::Lt),
        ("loop_start", Opcode::LoopStart),
        ("loop_end", Opcode::LoopEnd),
        ("print_num", Opcode::PrintNum),
        ("print_char", Opcode::PrintChar),
        ("read", Opcode::Read),
        ("store", Opcode::Store),
        ("load", Opcode::Load),
    ];
    for (name, expected) in cases {
        assert_eq!(parse_opcode(name).unwrap(), expected, "failed for {name}");
    }
}

#[test]
fn parse_opcode_case_insensitive() {
    assert_eq!(parse_opcode("ADD").unwrap(), Opcode::Add);
    assert_eq!(parse_opcode("Loop_Start").unwrap(), Opcode::LoopStart);
}

#[test]
fn parse_opcode_typo_suggests_dup() {
    let err = parse_opcode("dupe").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("dup"), "expected suggestion 'dup' in: {msg}");
}

#[test]
fn parse_opcode_unknown_no_suggestion() {
    let err = parse_opcode("xyz").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown opcode"), "expected 'unknown opcode' in: {msg}");
    // xyz is too far from any opcode to suggest
}

// ── Validation tests ──────────────────────────────────────────────

#[test]
fn validation_collects_all_errors() {
    let toml = r#"
        [metadata]
        name = "Bad"
        root_note = 60
        instrument = "piano"

        [values]
        range = [80, 40]

        [operations]
        60 = "dupe"
        62 = "xyz_bad"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    let result = validate(&raw);
    // Should have multiple errors: inverted range, unknown opcodes
    assert!(
        result.errors.len() >= 3,
        "expected >= 3 errors, got {}: {:?}",
        result.errors.len(),
        result.errors
    );
}

#[test]
fn validation_overlapping_exact_notes() {
    let toml = r#"
        [metadata]
        name = "Overlap"
        root_note = 60
        instrument = "piano"
        partial_coverage = true

        [operations]
        60 = "add"

        [[rules]]
        [rules.trigger]
        type = "note"
        note = 60

        [rules.emit]
        type = "op"
        opcode = "sub"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    let result = validate(&raw);
    let overlap_errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e, haydn_tuning::TuningError::Overlap { .. }))
        .collect();
    assert!(
        !overlap_errors.is_empty(),
        "expected at least one overlap error, got: {:?}",
        result.errors
    );
}

#[test]
fn validation_coverage_warning_when_opcodes_missing() {
    let toml = r#"
        [metadata]
        name = "Partial"
        root_note = 60
        instrument = "piano"

        [operations]
        60 = "add"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    let result = validate(&raw);
    assert!(
        !result.warnings.is_empty(),
        "expected coverage warnings"
    );
    assert!(result.warnings[0].contains("unmapped opcodes"));
}

#[test]
fn validation_no_coverage_warning_with_partial_flag() {
    let toml = r#"
        [metadata]
        name = "Partial OK"
        root_note = 60
        instrument = "piano"
        partial_coverage = true

        [operations]
        60 = "add"
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    let result = validate(&raw);
    assert!(
        result.warnings.is_empty(),
        "expected no coverage warnings with partial_coverage=true"
    );
}

#[test]
fn inverted_range_caught_in_validation() {
    let toml = r#"
        [metadata]
        name = "Inverted"
        root_note = 60
        instrument = "piano"
        partial_coverage = true

        [values]
        range = [80, 40]
    "#;
    let raw: RawTuningFile = toml::from_str(toml).unwrap();
    let result = validate(&raw);
    assert!(!result.errors.is_empty(), "expected validation error for inverted range");
}
