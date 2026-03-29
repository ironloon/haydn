use std::sync::mpsc;
use haydn::{MidiMsg, midi_callback, note_name, process_note, format_event_log, format_session_summary};

/// Build a raw MIDI NoteOn message (channel 0).
fn raw_note_on(note: u8, velocity: u8) -> Vec<u8> {
    vec![0x90, note, velocity]
}

/// Build a raw MIDI NoteOff message (channel 0).
fn raw_note_off(note: u8) -> Vec<u8> {
    vec![0x80, note, 0]
}

#[test]
fn midi_callback_parses_note_on() {
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();
    let msg = raw_note_on(60, 100);
    midi_callback(0, &msg, &mut tx);

    match rx.try_recv().unwrap() {
        MidiMsg::NoteOn { note, velocity } => {
            assert_eq!(note, 60);
            assert_eq!(velocity, 100);
        }
        other => panic!("Expected NoteOn, got {:?}", other),
    }
}

#[test]
fn midi_callback_parses_note_off() {
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();
    let msg = raw_note_off(60);
    midi_callback(0, &msg, &mut tx);

    match rx.try_recv().unwrap() {
        MidiMsg::NoteOff { note } => assert_eq!(note, 60),
        other => panic!("Expected NoteOff, got {:?}", other),
    }
}

#[test]
fn midi_callback_velocity_zero_is_note_off() {
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();
    let msg = raw_note_on(60, 0);
    midi_callback(0, &msg, &mut tx);

    match rx.try_recv().unwrap() {
        MidiMsg::NoteOff { note } => assert_eq!(note, 60),
        other => panic!("Expected NoteOff (vel=0 convention), got {:?}", other),
    }
}

#[test]
fn midi_callback_ignores_non_note_messages() {
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();
    // Control Change: channel 0, CC#1, value 64
    let msg = vec![0xB0, 1, 64];
    midi_callback(0, &msg, &mut tx);
    assert!(rx.try_recv().is_err(), "CC messages should be ignored");
}

#[test]
fn note_name_middle_c() {
    assert_eq!(note_name(60), "C4");
}

#[test]
fn note_name_covers_all_notes() {
    assert_eq!(note_name(69), "A4");
    assert_eq!(note_name(36), "C2");
    assert_eq!(note_name(59), "B3");
    assert_eq!(note_name(71), "B4");
    assert_eq!(note_name(0), "C-1");
    assert_eq!(note_name(127), "G9");
}

#[test]
fn process_note_push_value_through_piano_tuning() {
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Note 60 is the root note; notes 36-59 push values relative to root (60).
    // Note 48 should push 48 - 60 = -12
    let lines = process_note(48, 80, &mut engine, &mut vm);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Push(-12)"), "Expected Push(-12), got: {}", lines[0]);
    assert!(lines[0].contains("Stack: [-12]"), "Expected stack [-12], got: {}", lines[0]);
}

#[test]
fn process_note_push_then_add() {
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Push two values: note 63 → Push(3), note 65 → Push(5)... wait, 65=drop.
    // Use notes in value range (36-59): note 63 is above value range and is Op.
    // note 55 → Push(55-60) = Push(-5), note 57 → Push(57-60) = Push(-3)
    process_note(55, 80, &mut engine, &mut vm);
    process_note(57, 80, &mut engine, &mut vm);

    // note 60 = Add
    let lines = process_note(60, 80, &mut engine, &mut vm);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Op(Add)"), "Expected Add op, got: {}", lines[0]);
    assert!(lines[0].contains("Stack: [-8]"), "Expected stack [-8], got: {}", lines[0]);
}

#[test]
fn process_note_unmapped() {
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Note 120 is way outside any mapping range
    let lines = process_note(120, 64, &mut engine, &mut vm);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("(unmapped)"), "Expected unmapped, got: {}", lines[0]);
}

#[test]
fn process_note_dup_operation() {
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Push a value: note 50 → Push(50-60) = Push(-10)
    process_note(50, 80, &mut engine, &mut vm);

    // Note 64 = Dup
    let lines = process_note(64, 80, &mut engine, &mut vm);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Op(Dup)"), "Expected Dup, got: {}", lines[0]);
    assert!(lines[0].contains("Stack: [-10, -10]"), "Expected duplicated stack, got: {}", lines[0]);
}

#[test]
fn process_note_edge_case_empty_stack_default() {
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Add with empty stack → EmptyStackDefault (VM defaults missing values to 0)
    let lines = process_note(60, 80, &mut engine, &mut vm);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("EmptyStackDefault"), "Expected EmptyStackDefault, got: {}", lines[0]);
    assert!(lines[0].contains("Stack: [0]"), "Expected default result 0, got: {}", lines[0]);
}

#[test]
fn process_note_print_char_output() {
    // Test format_event_log with output directly (piano tuning only pushes negative values,
    // so constructing a printable char through the tuning would require many notes)
    let result = haydn::haydn_vm::StepResult {
        event: haydn::haydn_vm::Event::Op(haydn::haydn_vm::Opcode::PrintChar),
        operation: haydn::haydn_vm::Operation::Executed(haydn::haydn_vm::Opcode::PrintChar),
        stack_snapshot: vec![],
        output: Some(vec![65]), // 'A'
        edge_case: None,
    };
    let line = format_event_log(81, 80, &result);
    assert!(line.contains("Out: \"A\""), "Expected output 'A', got: {}", line);
}

#[test]
fn format_session_summary_empty_vm() {
    let vm = haydn::haydn_vm::HaydnVm::new();
    let summary = format_session_summary(&vm);
    assert!(summary.contains("Session Summary"));
    assert!(summary.contains("Final stack (0 items): []"));
}

#[test]
fn format_session_summary_with_stack() {
    let mut vm = haydn::haydn_vm::HaydnVm::new();
    vm.process_event(haydn::haydn_vm::Event::Push(42));
    vm.process_event(haydn::haydn_vm::Event::Push(7));
    let summary = format_session_summary(&vm);
    assert!(summary.contains("Final stack (2 items): [42, 7]"), "Got: {}", summary);
}

#[test]
fn full_pipeline_channel_to_vm() {
    // Simulate the exact path a real MIDI device takes:
    // raw bytes → midi_callback → channel → process_note
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Send two note-ons through the callback (value range notes)
    midi_callback(0, &raw_note_on(50, 100), &mut tx);
    midi_callback(0, &raw_note_on(55, 90), &mut tx);
    // Send Add operation
    midi_callback(0, &raw_note_on(60, 80), &mut tx);

    // Drain the channel and process each message
    let mut all_lines = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        match msg {
            MidiMsg::NoteOn { note, velocity } => {
                all_lines.extend(process_note(note, velocity, &mut engine, &mut vm));
            }
            MidiMsg::NoteOff { .. } => {}
        }
    }

    assert_eq!(all_lines.len(), 3);
    // note 50 → Push(50-60) = Push(-10)
    assert!(all_lines[0].contains("Push(-10)"), "got: {}", all_lines[0]);
    // note 55 → Push(55-60) = Push(-5)
    assert!(all_lines[1].contains("Push(-5)"), "got: {}", all_lines[1]);
    // note 60 → Add, stack: [-10, -5] → [-15]
    assert!(all_lines[2].contains("Op(Add)"), "got: {}", all_lines[2]);
    assert!(all_lines[2].contains("Stack: [-15]"), "got: {}", all_lines[2]);

    // Verify final VM state
    assert_eq!(vm.stack(), &[-15]);
}

#[test]
fn full_pipeline_with_note_off() {
    let (mut tx, rx) = mpsc::channel::<MidiMsg>();

    midi_callback(0, &raw_note_on(48, 100), &mut tx);
    midi_callback(0, &raw_note_off(48), &mut tx);

    let mut got_note_on = false;
    let mut got_note_off = false;
    while let Ok(msg) = rx.try_recv() {
        match msg {
            MidiMsg::NoteOn { note, .. } => {
                assert_eq!(note, 48);
                got_note_on = true;
            }
            MidiMsg::NoteOff { note } => {
                assert_eq!(note, 48);
                got_note_off = true;
            }
        }
    }
    assert!(got_note_on, "Should have received NoteOn");
    assert!(got_note_off, "Should have received NoteOff");
}

#[test]
fn vm_state_preserved_across_sessions() {
    // Simulates disconnect/reconnect: VM state persists when channel is recreated
    let mut engine = haydn::haydn_tuning::default_piano_tuning();
    let mut vm = haydn::haydn_vm::HaydnVm::new();

    // Session 1: push two values
    process_note(50, 80, &mut engine, &mut vm); // Push(-10)
    process_note(55, 80, &mut engine, &mut vm); // Push(-5)
    assert_eq!(vm.stack(), &[-10, -5]);

    // "Disconnect" — drop channel, create new one (vm and engine untouched)
    let (mut tx2, rx2) = mpsc::channel::<MidiMsg>();

    // Session 2: send Add through new channel
    midi_callback(0, &raw_note_on(60, 80), &mut tx2);
    if let Ok(MidiMsg::NoteOn { note, velocity }) = rx2.try_recv() {
        process_note(note, velocity, &mut engine, &mut vm);
    }

    // Stack state survived the "reconnect"
    assert_eq!(vm.stack(), &[-15]);
}
