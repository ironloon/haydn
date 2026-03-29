use crate::types::{NoteEvent, NoteSequence, RestEvent, ScoreEvent};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize, value};
use nom::multi::many0;
use nom::{IResult, Parser};
use std::time::Duration;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("invalid input at: {0}")]
    InvalidInput(String),
    #[error("MIDI note out of range: {0}")]
    MidiOutOfRange(i32),
}

/// Intermediate note representation before tie resolution.
#[derive(Debug, Clone)]
struct PartialNote {
    midi_note: u8,
    duration: Duration,
    tied: bool,
    velocity: u8,
}

/// Gradual expression mode — applied per note until cancelled.
#[derive(Debug, Clone, Copy, PartialEq)]
enum GradualMode {
    None,
    Crescendo,
    Decrescendo,
    Ritardando,
    Accelerando,
}

/// Parse state tracking default duration for carry-forward.
struct ParseState {
    default_duration: u32,
    bpm: f64,
    current_velocity: u8,
    gradual_mode: GradualMode,
    tempo_multiplier: f64,
}

impl ParseState {
    fn new(bpm: u32) -> Self {
        Self {
            default_duration: 4,
            bpm: bpm as f64,
            current_velocity: 80,
            gradual_mode: GradualMode::None,
            tempo_multiplier: 1.0,
        }
    }

    fn duration_to_ms(&self, value: u32, dotted: bool) -> Duration {
        let quarter_ms = 60_000.0 / self.bpm;
        let base_ms = quarter_ms * (4.0 / value as f64);
        let ms = if dotted { base_ms * 1.5 } else { base_ms };
        let ms = ms * self.tempo_multiplier;
        Duration::from_micros((ms * 1000.0) as u64)
    }

    /// Apply gradual changes per note (cresc/decresc adjust velocity, rit/accel adjust tempo).
    fn apply_gradual(&mut self) {
        match self.gradual_mode {
            GradualMode::Crescendo => {
                self.current_velocity = (self.current_velocity as u16 + 4).min(127) as u8;
            }
            GradualMode::Decrescendo => {
                self.current_velocity = (self.current_velocity as i16 - 4).max(16) as u8;
            }
            GradualMode::Ritardando => {
                self.tempo_multiplier *= 1.03; // each note ~3% slower
            }
            GradualMode::Accelerando => {
                self.tempo_multiplier *= 0.97; // each note ~3% faster
            }
            GradualMode::None => {}
        }
    }

    /// Map a dynamic marking to a velocity value.
    fn dynamic_to_velocity(dynamic: &str) -> Option<u8> {
        match dynamic {
            "ppp" => Some(16),
            "pp" => Some(33),
            "p" => Some(49),
            "mp" => Some(64),
            "mf" => Some(80),
            "f" => Some(96),
            "ff" => Some(112),
            "fff" => Some(127),
            "sfz" | "sf" => Some(120),
            "fp" => Some(96), // forte then piano — apply forte, next note reverts
            _ => None,
        }
    }
}

// --- Individual nom combinators ---

fn note_name(input: &str) -> IResult<&str, u8> {
    map(one_of("cdefgab"), |c| match c {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => unreachable!(),
    }).parse(input)
}

fn accidental(input: &str) -> IResult<&str, i8> {
    alt((
        value(2, tag("isis")),
        value(-2, tag("eses")),
        value(1, tag("is")),
        value(-1, tag("es")),
    )).parse(input)
}

fn octave_marks(input: &str) -> IResult<&str, i8> {
    let (input, marks): (_, Vec<char>) = many0(one_of("',")).parse(input)?;
    let offset: i8 = marks
        .iter()
        .map(|&c| if c == '\'' { 1i8 } else { -1i8 })
        .sum();
    Ok((input, offset))
}

fn duration_value(input: &str) -> IResult<&str, u32> {
    map(
        recognize(alt((
            tag("64"),
            tag("32"),
            tag("16"),
            tag("1"),
            tag("2"),
            tag("4"),
            tag("8"),
        ))),
        |s: &str| s.parse::<u32>().unwrap(),
    ).parse(input)
}

fn dot(input: &str) -> IResult<&str, bool> {
    map(opt(char('.')), |o| o.is_some()).parse(input)
}

fn tie(input: &str) -> IResult<&str, bool> {
    map(opt(char('~')), |o| o.is_some()).parse(input)
}

fn rest_event(input: &str) -> IResult<&str, (Option<u32>, bool)> {
    let (input, _) = char('r').parse(input)?;
    let (input, dur) = opt(duration_value).parse(input)?;
    let (input, d) = dot(input)?;
    Ok((input, (dur, d)))
}

fn note_full(input: &str) -> IResult<&str, (u8, i8, i8, Option<u32>, bool, bool)> {
    let (input, (semitone, acc, oct)) =
        (note_name, map(opt(accidental), |o: Option<i8>| o.unwrap_or(0)), octave_marks).parse(input)?;
    let (input, dur) = opt(duration_value).parse(input)?;
    let (input, d) = dot(input)?;
    let (input, t) = tie(input)?;
    Ok((input, (semitone, acc, oct, dur, d, t)))
}

fn barline(input: &str) -> IResult<&str, ()> {
    value((), char('|')).parse(input)
}

/// Compute MIDI note number from LilyPond absolute mode.
/// c (no marks) = C3 (MIDI 48), c' = C4 (MIDI 60), c'' = C5 (MIDI 72), c, = C2 (MIDI 36)
fn compute_midi(semitone: u8, accidental: i8, octave_offset: i8) -> Result<u8, ParseError> {
    let midi = 48i32 + semitone as i32 + accidental as i32 + (octave_offset as i32 * 12);
    if (0..=127).contains(&midi) {
        Ok(midi as u8)
    } else {
        Err(ParseError::MidiOutOfRange(midi))
    }
}

fn skip_ws_and_comments(mut input: &str) -> &str {
    loop {
        let trimmed = input.trim_start();
        if trimmed.starts_with('%') {
            match trimmed.find('\n') {
                Some(pos) => input = &trimmed[pos + 1..],
                None => return "",
            }
        } else {
            return trimmed;
        }
    }
}

#[derive(Debug, Clone)]
enum PartialOrRest {
    Note(PartialNote),
    Rest(Duration),
}

fn resolve_ties(partials: &[PartialOrRest]) -> Result<NoteSequence, ParseError> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < partials.len() {
        match &partials[i] {
            PartialOrRest::Rest(duration) => {
                result.push(ScoreEvent::Rest(RestEvent {
                    duration: *duration,
                }));
                i += 1;
            }
            PartialOrRest::Note(note) => {
                let mut total_duration = note.duration;
                let midi = note.midi_note;
                let velocity = note.velocity;
                let mut tied = note.tied;

                while tied {
                    i += 1;
                    if i >= partials.len() {
                        break;
                    }
                    match &partials[i] {
                        PartialOrRest::Note(next) if next.midi_note == midi => {
                            total_duration += next.duration;
                            tied = next.tied;
                        }
                        _ => {
                            tied = false;
                            break;
                        }
                    }
                }

                result.push(ScoreEvent::Note(NoteEvent {
                    midi_note: midi,
                    duration: total_duration,
                    velocity,
                }));

                i += 1;
            }
        }
    }

    Ok(result)
}

/// Parse a LilyPond input string into a NoteSequence.
///
/// Uses absolute mode: `c` = C3 (MIDI 48), `c'` = C4 (MIDI 60).
/// Duration carry-forward: notes without explicit duration inherit from previous.
/// Ties combine durations of consecutive notes with the same pitch.
///
/// Supported directives (LilyPond-style):
/// - Dynamics: `\ppp`, `\pp`, `\p`, `\mp`, `\mf`, `\f`, `\ff`, `\fff`, `\sfz`
/// - Gradual dynamics: `\cresc`, `\decresc`, `\dim`
/// - Tempo: `\tempo N` (changes BPM mid-stream)
/// - Gradual tempo: `\rit`, `\accel`, `\atempo` (resets tempo multiplier)
pub fn parse(input: &str, bpm: u32) -> Result<NoteSequence, ParseError> {
    let mut state = ParseState::new(bpm);
    let mut partials: Vec<PartialOrRest> = Vec::new();
    let mut remaining = input;
    let mut fp_pending = false; // for \fp: next note after forte reverts to piano

    loop {
        remaining = skip_ws_and_comments(remaining);

        if remaining.is_empty() {
            break;
        }

        // Try barline
        if let Ok((rest, _)) = barline(remaining) {
            remaining = rest;
            continue;
        }

        // Try directive (\command)
        if remaining.starts_with('\\') {
            let directive_text = &remaining[1..];
            // Extract the command word
            let end = directive_text
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(directive_text.len());
            let command = &directive_text[..end];
            remaining = &directive_text[end..];

            // Check for dynamics
            if let Some(vel) = ParseState::dynamic_to_velocity(command) {
                if command == "fp" {
                    state.current_velocity = vel; // forte for this note
                    fp_pending = true;
                } else {
                    state.current_velocity = vel;
                    // Explicit dynamic cancels gradual dynamic changes
                    if matches!(
                        state.gradual_mode,
                        GradualMode::Crescendo | GradualMode::Decrescendo
                    ) {
                        state.gradual_mode = GradualMode::None;
                    }
                }
                continue;
            }

            match command {
                "tempo" => {
                    // Parse BPM number after \tempo
                    let after = remaining.trim_start();
                    let num_end = after
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(after.len());
                    if num_end > 0 {
                        if let Ok(new_bpm) = after[..num_end].parse::<u32>() {
                            state.bpm = new_bpm as f64;
                            state.tempo_multiplier = 1.0; // reset tempo multiplier on explicit tempo
                            state.gradual_mode = match state.gradual_mode {
                                GradualMode::Ritardando | GradualMode::Accelerando => {
                                    GradualMode::None
                                }
                                other => other,
                            };
                        }
                        remaining = &after[num_end..];
                    }
                }
                "cresc" | "crescendo" => {
                    state.gradual_mode = GradualMode::Crescendo;
                }
                "decresc" | "decrescendo" | "dim" => {
                    state.gradual_mode = GradualMode::Decrescendo;
                }
                "rit" | "ritardando" => {
                    state.gradual_mode = GradualMode::Ritardando;
                }
                "accel" | "accelerando" => {
                    state.gradual_mode = GradualMode::Accelerando;
                }
                "atempo" | "a_tempo" => {
                    state.tempo_multiplier = 1.0;
                    state.gradual_mode = match state.gradual_mode {
                        GradualMode::Ritardando | GradualMode::Accelerando => GradualMode::None,
                        other => other,
                    };
                }
                _ => {
                    // Unknown directive — skip silently (LilyPond has many we don't support)
                }
            }
            continue;
        }

        // Try rest
        if let Ok((rest, (dur, dotted))) = rest_event(remaining) {
            let dur_val = dur.unwrap_or(state.default_duration);
            if dur.is_some() {
                state.default_duration = dur_val;
            }
            partials.push(PartialOrRest::Rest(state.duration_to_ms(dur_val, dotted)));
            // Rests contribute to gradual tempo changes
            if matches!(
                state.gradual_mode,
                GradualMode::Ritardando | GradualMode::Accelerando
            ) {
                state.apply_gradual();
            }
            remaining = rest;
            continue;
        }

        // Try note
        if let Ok((rest, (semitone, acc, oct, dur, dotted, tied))) = note_full(remaining) {
            let dur_val = dur.unwrap_or(state.default_duration);
            if dur.is_some() {
                state.default_duration = dur_val;
            }
            let midi = compute_midi(semitone, acc, oct).map_err(|_| {
                ParseError::InvalidInput(remaining[..remaining.len().min(20)].to_string())
            })?;
            let velocity = state.current_velocity;
            let duration = state.duration_to_ms(dur_val, dotted);
            partials.push(PartialOrRest::Note(PartialNote {
                midi_note: midi,
                duration,
                tied,
                velocity,
            }));
            state.apply_gradual();
            if fp_pending {
                state.current_velocity = 49; // revert to piano after \fp note
                fp_pending = false;
            }
            remaining = rest;
            continue;
        }

        return Err(ParseError::InvalidInput(
            remaining[..remaining.len().min(20)].to_string(),
        ));
    }

    resolve_ties(&partials)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const BPM: u32 = 120;

    fn quarter() -> Duration {
        Duration::from_millis(500)
    }

    fn dotted_quarter() -> Duration {
        Duration::from_millis(750)
    }

    fn half() -> Duration {
        Duration::from_millis(1000)
    }

    #[test]
    fn test_note_name_parser() {
        assert_eq!(note_name("c rest"), Ok((" rest", 0)));
        assert_eq!(note_name("g'"), Ok(("'", 7)));
    }

    #[test]
    fn test_accidental_parser() {
        assert_eq!(accidental("is'"), Ok(("'", 1)));
        assert_eq!(accidental("es4"), Ok(("4", -1)));
        assert_eq!(accidental("isis"), Ok(("", 2)));
        assert_eq!(accidental("eses"), Ok(("", -2)));
    }

    #[test]
    fn test_octave_marks_parser() {
        assert_eq!(octave_marks("''4"), Ok(("4", 2)));
        assert_eq!(octave_marks(",,"), Ok(("", -2)));
        assert_eq!(octave_marks("4"), Ok(("4", 0)));
    }

    #[test]
    fn test_compute_midi() {
        assert_eq!(compute_midi(0, 0, 0).unwrap(), 48);
        assert_eq!(compute_midi(0, 0, 1).unwrap(), 60);
        assert_eq!(compute_midi(0, 0, 2).unwrap(), 72);
        assert_eq!(compute_midi(0, 0, -1).unwrap(), 36);
        assert_eq!(compute_midi(0, 1, 1).unwrap(), 61);
        assert_eq!(compute_midi(11, -1, 1).unwrap(), 70);
        assert_eq!(compute_midi(2, 0, 1).unwrap(), 62);
    }

    #[test]
    fn test_parse_single_note() {
        let result = parse("c4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 48);
                assert_eq!(n.duration, quarter());
            }
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_parse_c_prime_scale() {
        let result = parse("c' d' e' f' g' a' b'", BPM).unwrap();
        assert_eq!(result.len(), 7);
        let expected_midi = [60, 62, 64, 65, 67, 69, 71];
        for (i, event) in result.iter().enumerate() {
            match event {
                ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected_midi[i], "note {}", i),
                _ => panic!("expected note at {}", i),
            }
        }
    }

    #[test]
    fn test_parse_two_octaves_up() {
        let result = parse("c''4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 72),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_parse_octave_down() {
        let result = parse("c,4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 36),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_parse_sharp() {
        let result = parse("cis'4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 61),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_parse_flat() {
        let result = parse("bes'4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 70),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_parse_dotted_note() {
        let result = parse("c'4.", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 60);
                assert_eq!(n.duration, dotted_quarter());
            }
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_duration_carry_forward() {
        let result = parse("c'4 d'", BPM).unwrap();
        assert_eq!(result.len(), 2);
        match (&result[0], &result[1]) {
            (ScoreEvent::Note(a), ScoreEvent::Note(b)) => {
                assert_eq!(a.duration, quarter());
                assert_eq!(b.duration, quarter());
                assert_eq!(b.midi_note, 62);
            }
            _ => panic!("expected two notes"),
        }
    }

    #[test]
    fn test_parse_rest() {
        let result = parse("r4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Rest(r) => assert_eq!(r.duration, quarter()),
            _ => panic!("expected rest"),
        }
    }

    #[test]
    fn test_barline_ignored() {
        let result = parse("c'4 d' e' f' |", BPM).unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_tie_combines_durations() {
        let result = parse("c'4~ c'4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 60);
                assert_eq!(n.duration, half());
            }
            _ => panic!("expected single tied note"),
        }
    }

    #[test]
    fn test_comment_ignored() {
        let result = parse("% comment\nc'4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 60),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_empty_input() {
        let result = parse("", BPM).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_invalid_note_name() {
        let result = parse("xyz", BPM);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse("   \n\t  ", BPM).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_only_comments() {
        let result = parse("% just a comment\n% another one", BPM).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_only_barlines() {
        let result = parse("| | |", BPM).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_mixed_score() {
        let result = parse("c'4 d'8 r4 e'2 |", BPM).unwrap();
        assert_eq!(result.len(), 4);
        match &result[0] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 60);
                assert_eq!(n.duration, quarter());
            }
            _ => panic!("expected note"),
        }
        match &result[1] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 62);
                assert_eq!(n.duration, Duration::from_millis(250));
            }
            _ => panic!("expected note"),
        }
        match &result[2] {
            ScoreEvent::Rest(r) => assert_eq!(r.duration, quarter()),
            _ => panic!("expected rest"),
        }
        match &result[3] {
            ScoreEvent::Note(n) => {
                assert_eq!(n.midi_note, 64);
                assert_eq!(n.duration, half());
            }
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_piano_tuning_value_zone() {
        let result = parse("c,4 d, e, f, g, a, b,", BPM).unwrap();
        assert_eq!(result.len(), 7);
        let expected = [36, 38, 40, 41, 43, 45, 47];
        for (i, event) in result.iter().enumerate() {
            match event {
                ScoreEvent::Note(n) => assert_eq!(n.midi_note, expected[i], "note {}", i),
                _ => panic!("expected note at {}", i),
            }
        }
    }

    #[test]
    fn test_whole_note_duration() {
        let result = parse("c'1", BPM).unwrap();
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(2000)),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_sixteenth_note_duration() {
        let result = parse("c'16", BPM).unwrap();
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(125)),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_dynamic_forte() {
        let result = parse("\\f c'4 d'4", BPM).unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.velocity, 96),
            _ => panic!("expected note"),
        }
        match &result[1] {
            ScoreEvent::Note(n) => assert_eq!(n.velocity, 96),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_dynamic_piano() {
        let result = parse("\\p c'4", BPM).unwrap();
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.velocity, 49),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_dynamic_change_mid_stream() {
        let result = parse("\\p c'4 \\f d'4", BPM).unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.velocity, 49),
            _ => panic!("expected note"),
        }
        match &result[1] {
            ScoreEvent::Note(n) => assert_eq!(n.velocity, 96),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_crescendo_increases_velocity() {
        let result = parse("\\p \\cresc c'4 d' e' f'", BPM).unwrap();
        assert_eq!(result.len(), 4);
        let velocities: Vec<u8> = result
            .iter()
            .map(|e| match e {
                ScoreEvent::Note(n) => n.velocity,
                _ => 0,
            })
            .collect();
        // First note at piano (49), then each gets +4
        assert_eq!(velocities[0], 49);
        assert!(velocities[1] > velocities[0]);
        assert!(velocities[2] > velocities[1]);
        assert!(velocities[3] > velocities[2]);
    }

    #[test]
    fn test_decrescendo_decreases_velocity() {
        let result = parse("\\f \\decresc c'4 d' e' f'", BPM).unwrap();
        let velocities: Vec<u8> = result
            .iter()
            .map(|e| match e {
                ScoreEvent::Note(n) => n.velocity,
                _ => 0,
            })
            .collect();
        assert_eq!(velocities[0], 96);
        assert!(velocities[1] < velocities[0]);
        assert!(velocities[2] < velocities[1]);
    }

    #[test]
    fn test_tempo_change() {
        // At 120 BPM a quarter = 500ms, at 60 BPM a quarter = 1000ms
        let result = parse("c'4 \\tempo 60 d'4", BPM).unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(500)),
            _ => panic!("expected note"),
        }
        match &result[1] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(1000)),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_ritardando_slows_notes() {
        let result = parse("\\rit c'4 d' e' f'", BPM).unwrap();
        let durations: Vec<Duration> = result
            .iter()
            .map(|e| match e {
                ScoreEvent::Note(n) => n.duration,
                ScoreEvent::Rest(r) => r.duration,
            })
            .collect();
        // Each note should be progressively longer
        assert!(durations[1] > durations[0]);
        assert!(durations[2] > durations[1]);
        assert!(durations[3] > durations[2]);
    }

    #[test]
    fn test_atempo_resets_ritardando() {
        let result = parse("\\rit c'4 d' \\atempo e'4 f'", BPM).unwrap();
        match &result[2] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(500)),
            _ => panic!("expected note"),
        }
        match &result[3] {
            ScoreEvent::Note(n) => assert_eq!(n.duration, Duration::from_millis(500)),
            _ => panic!("expected note"),
        }
    }

    #[test]
    fn test_unknown_directive_ignored() {
        let result = parse("\\relative c'4", BPM).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ScoreEvent::Note(n) => assert_eq!(n.midi_note, 60), // c' = C4
            _ => panic!("expected note"),
        }
    }
}
