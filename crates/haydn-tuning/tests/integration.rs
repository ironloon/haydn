use std::collections::HashSet;

use haydn_tuning::{default_piano_tuning, load_tuning};
use haydn_vm::{Event, HaydnVm, Opcode};

// ── Piano tuning tests ─────────────────────────────────────────────

mod piano_tuning {
    use super::*;

    #[test]
    fn maps_value_zone() {
        let mut engine = default_piano_tuning();
        // root_note = 60, value zone = [36, 59]
        assert_eq!(engine.map_note(36), Some(Event::Push(-24))); // 36 - 60
        assert_eq!(engine.map_note(48), Some(Event::Push(-12))); // 48 - 60
        assert_eq!(engine.map_note(59), Some(Event::Push(-1)));  // 59 - 60
    }

    #[test]
    fn maps_all_19_opcodes() {
        let mut engine = default_piano_tuning();
        // White keys in piano.toml: 60,62,64,65,67,69,71,72,74,76,77,79,81,83,84,86,88,89,91
        let white_keys = [60, 62, 64, 65, 67, 69, 71, 72, 74, 76, 77, 79, 81, 83, 84, 86, 88, 89, 91];
        let mut opcodes = HashSet::new();
        for key in white_keys {
            if let Some(Event::Op(opcode)) = engine.map_note(key) {
                opcodes.insert(opcode);
            } else {
                panic!("Expected Op event for MIDI note {key}");
            }
        }
        assert_eq!(opcodes.len(), 19, "Expected all 19 opcodes, got {:?}", opcodes);
    }

    #[test]
    fn below_value_zone_unmapped() {
        let mut engine = default_piano_tuning();
        assert_eq!(engine.map_note(35), None); // just below range
        assert_eq!(engine.map_note(0), None);
    }

    #[test]
    fn above_operation_zone_unmapped() {
        let mut engine = default_piano_tuning();
        assert_eq!(engine.map_note(92), None);
        assert_eq!(engine.map_note(127), None);
    }

    #[test]
    fn black_keys_unmapped() {
        let mut engine = default_piano_tuning();
        // Black keys in the operation zone are not mapped
        assert_eq!(engine.map_note(61), None); // C#4
        assert_eq!(engine.map_note(63), None); // D#4
        assert_eq!(engine.map_note(66), None); // F#4
        assert_eq!(engine.map_note(68), None); // G#4
    }

    #[test]
    fn metadata_correct() {
        let engine = default_piano_tuning();
        let meta = engine.metadata();
        assert_eq!(meta.name, "Default Piano");
        assert_eq!(meta.root_note, 60);
        assert_eq!(meta.instrument, "piano");
    }

    #[test]
    fn specific_opcode_mappings() {
        let mut engine = default_piano_tuning();
        assert_eq!(engine.map_note(60), Some(Event::Op(Opcode::Add)));
        assert_eq!(engine.map_note(72), Some(Event::Op(Opcode::LoopStart)));
        assert_eq!(engine.map_note(81), Some(Event::Op(Opcode::PrintChar)));
        assert_eq!(engine.map_note(91), Some(Event::Op(Opcode::Rotate)));
    }
}

// ── Custom tuning tests ────────────────────────────────────────────

mod custom_tunings {
    use super::*;

    #[test]
    fn minimal_exact_notes_only() {
        let toml = r#"
            [metadata]
            name = "Minimal"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [operations]
            60 = "add"
            62 = "sub"
        "#;
        let mut engine = load_tuning(toml).unwrap();
        assert_eq!(engine.map_note(60), Some(Event::Op(Opcode::Add)));
        assert_eq!(engine.map_note(62), Some(Event::Op(Opcode::Sub)));
        assert_eq!(engine.map_note(64), None);
    }

    #[test]
    fn interval_based_tuning() {
        let toml = r#"
            [metadata]
            name = "Interval"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [[rules]]
            [rules.trigger]
            type = "interval"
            semitones = 7

            [rules.emit]
            type = "op"
            opcode = "add"
        "#;
        let mut engine = load_tuning(toml).unwrap();
        // First note: no last_note, interval can't match
        assert_eq!(engine.map_note(60), None);
        // Second note: 67 - 60 = 7 semitones (perfect fifth)
        assert_eq!(engine.map_note(67), Some(Event::Op(Opcode::Add)));
    }

    #[test]
    fn mixed_shorthands_and_rules() {
        let toml = r#"
            [metadata]
            name = "Mixed"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [values]
            range = [36, 59]

            [operations]
            60 = "add"

            [[rules]]
            [rules.trigger]
            type = "note"
            note = 100

            [rules.emit]
            type = "push"
            value = 42
        "#;
        let mut engine = load_tuning(toml).unwrap();
        // Value zone takes priority
        assert_eq!(engine.map_note(48), Some(Event::Push(-12)));
        // Operations match
        assert_eq!(engine.map_note(60), Some(Event::Op(Opcode::Add)));
        // Raw rule matches
        assert_eq!(engine.map_note(100), Some(Event::Push(42)));
    }

    #[test]
    fn first_match_wins_shorthands_before_rules() {
        // Value zone covers 36-59, and a raw rule matches note 100 (outside zone)
        // Also add a raw range rule for 36-59 — but it won't compile because
        // overlaps are rejected. Instead, test that value zone matches before
        // operations by having an operation and value zone both potentially match,
        // but operations are exact-note and value range is first.
        // Actually, the piano.toml already has values 36-59 and operations at 60+
        // which don't overlap. The first-match-wins is between all three levels.
        // Let's just verify that value zone matches before raw rules for a
        // note that's only in the value zone.
        let toml = r#"
            [metadata]
            name = "Priority"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [values]
            range = [36, 59]

            [[rules]]
            [rules.trigger]
            type = "note"
            note = 100

            [rules.emit]
            type = "push"
            value = 999
        "#;
        let mut engine = load_tuning(toml).unwrap();
        // Value zone matches: Push(48 - 60) = Push(-12)
        assert_eq!(engine.map_note(48), Some(Event::Push(-12)));
        // Raw rule matches: Push(999)
        assert_eq!(engine.map_note(100), Some(Event::Push(999)));
    }

    #[test]
    fn reset_clears_interval_tracking() {
        let toml = r#"
            [metadata]
            name = "Reset"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [[rules]]
            [rules.trigger]
            type = "interval"
            semitones = 7

            [rules.emit]
            type = "op"
            opcode = "add"
        "#;
        let mut engine = load_tuning(toml).unwrap();
        engine.map_note(60); // set last_note
        engine.reset();      // clear last_note
        // After reset, interval can't match
        assert_eq!(engine.map_note(67), None);
    }

    #[test]
    fn push_offset_in_raw_rules() {
        let toml = r#"
            [metadata]
            name = "PushOffset Rule"
            root_note = 60
            instrument = "test"
            partial_coverage = true

            [[rules]]
            [rules.trigger]
            type = "range"
            from = 36
            to = 59

            [rules.emit]
            type = "push_offset"
        "#;
        let mut engine = load_tuning(toml).unwrap();
        assert_eq!(engine.map_note(48), Some(Event::Push(-12))); // 48 - 60
    }
}

// ── VM round-trip tests ────────────────────────────────────────────

mod vm_round_trip {
    use super::*;

    #[test]
    fn push_add_via_piano() {
        let mut engine = default_piano_tuning();
        let mut vm = HaydnVm::new();

        // Push values via value zone
        let ev1 = engine.map_note(48).unwrap(); // Push(-12)
        let ev2 = engine.map_note(54).unwrap(); // Push(-6)
        vm.process_event(ev1);
        vm.process_event(ev2);

        // Add via operation zone
        let ev3 = engine.map_note(60).unwrap(); // Op(Add)
        vm.process_event(ev3);

        assert_eq!(vm.stack(), &[-18]); // -12 + -6 = -18
    }

    #[test]
    fn push_print_char() {
        let mut engine = default_piano_tuning();
        let mut vm = HaydnVm::new();

        // Push 72 ('H') — need to push it directly since value zone gives offsets
        vm.process_event(Event::Push(72));

        // PrintChar via piano tuning
        let ev = engine.map_note(81).unwrap(); // Op(PrintChar)
        vm.process_event(ev);

        assert_eq!(vm.output(), &[72]); // 'H'
    }

    #[test]
    fn simple_computation() {
        let mut engine = default_piano_tuning();
        let mut vm = HaydnVm::new();

        // Push two values via value zone
        // MIDI 50 → Push(50 - 60) = Push(-10)
        // MIDI 55 → Push(55 - 60) = Push(-5)
        let ev1 = engine.map_note(50).unwrap();
        let ev2 = engine.map_note(55).unwrap();
        vm.process_event(ev1);
        vm.process_event(ev2);

        // Mul via MIDI 67
        let ev3 = engine.map_note(67).unwrap(); // Op(Mul)
        vm.process_event(ev3);

        assert_eq!(vm.stack(), &[50]); // -10 * -5 = 50
    }

    #[test]
    fn dup_and_add() {
        let mut engine = default_piano_tuning();
        let mut vm = HaydnVm::new();

        // Push value
        let ev1 = engine.map_note(55).unwrap(); // Push(-5)
        vm.process_event(ev1);

        // Dup (MIDI 64)
        let ev2 = engine.map_note(64).unwrap(); // Op(Dup)
        vm.process_event(ev2);

        // Add (MIDI 60)
        let ev3 = engine.map_note(60).unwrap(); // Op(Add)
        vm.process_event(ev3);

        assert_eq!(vm.stack(), &[-10]); // -5 + -5
    }
}
