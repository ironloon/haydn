use haydn_performer::parser;
use haydn_performer::types::ScoreEvent;
use std::time::Duration;

const BPM: u32 = 120;

#[test]
fn test_c_major_scale_octave_4() {
    let result = parser::parse("c'4 d' e' f' g' a' b'", BPM).unwrap();
    assert_eq!(result.len(), 7);
    let expected = [60, 62, 64, 65, 67, 69, 71];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected[i]),
            _ => panic!("expected note"),
        }
    }
}

#[test]
fn test_chromatic_accidentals() {
    let result = parser::parse("cis'4 dis' fis' gis' ais'", BPM).unwrap();
    let expected = [61, 63, 66, 68, 70];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected[i], "accidental {}", i),
            _ => panic!("expected note"),
        }
    }
}

#[test]
fn test_flats() {
    let result = parser::parse("bes'4 ees' aes'", BPM).unwrap();
    let expected = [70, 63, 68];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected[i], "flat {}", i),
            _ => panic!("expected note"),
        }
    }
}

#[test]
fn test_octave_spanning() {
    let result = parser::parse("c,,4 c, c c' c'' c'''", BPM).unwrap();
    let expected = [24, 36, 48, 60, 72, 84];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected[i], "octave {}", i),
            _ => panic!("expected note"),
        }
    }
}

#[test]
fn test_duration_variety() {
    let result = parser::parse("c'1 c'2 c'4 c'8 c'16 c'32", BPM).unwrap();
    let expected_ms = [2000, 1000, 500, 250, 125, 62];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => {
                let ms = n.duration.as_millis();
                assert!(
                    (ms as i64 - expected_ms[i] as i64).unsigned_abs() <= 1,
                    "duration {}: got {}ms, expected {}ms",
                    i,
                    ms,
                    expected_ms[i]
                );
            }
            _ => panic!("expected note"),
        }
    }
}

#[test]
fn test_rests_in_melody() {
    let result = parser::parse("c'4 r4 d'4 r8 e'4", BPM).unwrap();
    assert_eq!(result.len(), 5);
    assert!(matches!(&result[0], ScoreEvent::Note(_)));
    assert!(matches!(&result[1], ScoreEvent::Rest(_)));
    assert!(matches!(&result[2], ScoreEvent::Note(_)));
    assert!(matches!(&result[3], ScoreEvent::Rest(_)));
    assert!(matches!(&result[4], ScoreEvent::Note(_)));
}

#[test]
fn test_ties_across_barlines() {
    let result = parser::parse("c'4 d' e' f'~ | f' g' a' b'", BPM).unwrap();
    assert_eq!(result.len(), 7); // f' tied = 1 event, not 2
    match &result[3] {
        ScoreEvent::Note(n) => {
            assert_eq!(n.midi_note, 65); // F4
            assert_eq!(n.duration, Duration::from_millis(1000)); // 500+500
        }
        _ => panic!("expected tied note"),
    }
}

#[test]
fn test_comments_between_notes() {
    let input = "c'4 % first note\nd'4 % second note\ne'4";
    let result = parser::parse(input, BPM).unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_dotted_rhythms() {
    let result = parser::parse("c'4. d'8 e'2.", BPM).unwrap();
    assert_eq!(result.len(), 3);
    match &result[0] {
        ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(750)),
        _ => panic!("expected note"),
    }
    match &result[1] {
        ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(250)),
        _ => panic!("expected note"),
    }
    match &result[2] {
        ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(1500)),
        _ => panic!("expected note"),
    }
}

#[test]
fn test_full_piece() {
    // Twinkle Twinkle Little Star - first phrase
    let input = "c'4 c' g' g' | a' a' g'2 | f'4 f' e' e' | d' d' c'2";
    let result = parser::parse(input, BPM).unwrap();
    assert_eq!(result.len(), 14);

    // Verify melody
    let expected_midi = [60, 60, 67, 67, 69, 69, 67, 65, 65, 64, 64, 62, 62, 60];
    for (i, event) in result.iter().enumerate() {
        match event {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected_midi[i], "note {}", i),
            _ => panic!("expected note at {}", i),
        }
    }
}
